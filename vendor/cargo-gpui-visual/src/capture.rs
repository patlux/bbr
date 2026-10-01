//! Capture and seal a fresh run without comparing, reviewing or accepting it.
//!
//! These are the shared CLI producer semantics, not a test runner or renderer.
//! Producer/probe processes have deadlines and group cleanup. Host GUI/GPU
//! exclusion still belongs to the caller; never run an unowned native producer.
use crate::{
    config::Config,
    process::{self, Limits},
    store::{self, Manifest},
};
use anyhow::{ensure, Context, Result};
use std::{
    fs,
    path::Path,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

/// Bounded producer/probe lifetimes. Callers retain host GUI/GPU gate ownership.
#[derive(Clone, Debug)]
pub struct Execution {
    pub producer: Limits,
    pub environment: Limits,
}
impl Default for Execution {
    fn default() -> Self {
        Self {
            producer: Limits::default(),
            environment: Limits {
                timeout: Duration::from_secs(30),
                ..Limits::default()
            },
        }
    }
}

fn environment(
    args: &[String],
    repo: &Path,
    execution: &Execution,
    execute: &mut impl FnMut(&mut Command, &Limits, bool) -> Result<process::Output>,
) -> Result<serde_json::Value> {
    ensure!(!args.is_empty(), "Environment command is empty");
    let output = execute(
        Command::new(&args[0])
            .args(&args[1..])
            .current_dir(repo)
            .env_remove(crate::selection::ENVIRONMENT),
        &execution.environment,
        true,
    )?;
    if !output.status.success() {
        return Err(crate::diagnostic::Code::EnvironmentProbeFailed
            .error("Environment probe failed")
            .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn invoke(
    config: &Config,
    repo: &Path,
    run: &Path,
    execution: &Execution,
    execute: &mut impl FnMut(&mut Command, &Limits, bool) -> Result<process::Output>,
) -> Result<()> {
    ensure!(!config.capture.is_empty(), "Capture command is empty");
    let replaced: Vec<_> = config
        .capture
        .iter()
        .map(|a| a.replace("{run}", &run.to_string_lossy()))
        .collect();
    let mut command = Command::new(&replaced[0]);
    command
        .args(&replaced[1..])
        .current_dir(repo)
        .env("GPUI_VISUAL_OUTPUT", run);
    // Never inherit a stale selection into a supposedly complete capture.
    command.env_remove(crate::selection::ENVIRONMENT);
    if let Some(selection) = &config.selection {
        command.env(
            crate::selection::ENVIRONMENT,
            serde_json::to_string(selection)?,
        );
    }
    let status = execute(&mut command, &execution.producer, false)?.status;
    if !status.success() {
        return Err(crate::diagnostic::Code::ProducerFailed
            .error(format!("Capture failed: {status}"))
            .into());
    }
    Ok(())
}

/// Invoke a producer for an explicit new run, then validate its complete seal.
///
/// Relative paths resolve against `repo`, never the caller's working directory.
/// `baselines` and `report` are the caller's effective paths (including CLI
/// overrides): they protect disjointness and the exact source-policy exclusion.
/// Neither directory is written. Partial failed runs remain as diagnostic
/// evidence; they are never retried, overwritten, compared or accepted here.
/// Existing sealed producers must honor the configured policy and source hashes.
/// Raw producers must supply the full paired inventory and may not self-seal.
/// The producer inventory is not an independent required-coverage authority.
///
/// Default producer deadline: 30 minutes; each environment probe: 30 seconds.
/// The caller must hold the required host GUI/GPU gate. Use `capture_with` for
/// explicit process limits and cancellation; the library installs no handlers.
pub fn capture(
    repo: &Path,
    config: &Config,
    run: &Path,
    baselines: &Path,
    report: &Path,
) -> Result<Manifest> {
    capture_with(
        repo,
        config,
        run,
        baselines,
        report,
        &Execution::default(),
        &AtomicBool::new(false),
    )
}

/// The same capture authority with caller-owned cancellation and process limits.
pub fn capture_with(
    repo: &Path,
    config: &Config,
    run: &Path,
    baselines: &Path,
    report: &Path,
    execution: &Execution,
    cancelled: &AtomicBool,
) -> Result<Manifest> {
    capture_timed(
        Paths {
            repo,
            run,
            baselines,
            report,
        },
        config,
        execution,
        cancelled,
        &mut crate::timing::Timings::default(),
    )
}

/// Explicit effective paths shared by guarded orchestration and the CLI.
#[derive(Clone, Copy)]
pub struct Paths<'a> {
    pub repo: &'a Path,
    pub run: &'a Path,
    pub baselines: &'a Path,
    pub report: &'a Path,
}

/// Hold the durable host lease and compatibility flocks across capture, probes,
/// validation and descendant cleanup. Failed capture is not a dirty host if all
/// process cleanup is proven; failed cleanup leaves an unfinished lease.
#[cfg(unix)]
pub fn capture_guarded(
    paths: Paths<'_>,
    config: &Config,
    execution: &Execution,
    cancelled: &AtomicBool,
    options: &crate::gate::Options,
) -> Result<Manifest> {
    capture_guarded_timed(
        paths,
        config,
        execution,
        cancelled,
        options,
        &mut crate::timing::Timings::default(),
    )
}

/// Guarded capture with observational phase times, including gate wait and cleanup.
#[cfg(unix)]
pub fn capture_guarded_timed(
    paths: Paths<'_>,
    config: &Config,
    execution: &Execution,
    cancelled: &AtomicBool,
    options: &crate::gate::Options,
    timings: &mut crate::timing::Timings,
) -> Result<Manifest> {
    let mut gate = timings.measure("host_gate_wait_ms", || {
        crate::gate::Gate::acquire(options, cancelled)
    })?;
    let started = std::time::Instant::now();
    let result = capture_inner(
        paths,
        config,
        execution,
        cancelled,
        &mut |command, limits, stdout| {
            timings.measure(
                if stdout {
                    "environment_probe_ms"
                } else {
                    "producer_ms"
                },
                || gate.execute(command, limits, cancelled, stdout),
            )
        },
    );
    timings.record("capture_total_ms", started);
    if result
        .as_ref()
        .err()
        .is_some_and(|error| error.downcast_ref::<process::CleanupFailure>().is_some())
    {
        // Includes failed inventory Git cleanup, which never clears this lease.
        return result;
    }
    if let Err(error) = timings.measure("host_gate_finish_ms", || gate.finish()) {
        anyhow::bail!(
            "Host lease not clean: {error:#}; capture error: {:?}",
            result.err()
        );
    }
    result
}

/// Capture a raw consumer producer in the caller's process, sharing exactly the
/// external-command source/environment/image seal checks. This avoids nesting
/// process-group supervisors around a consumer that itself owns bounded builds,
/// native producer groups and a durable host lease. The caller must use the
/// portable process/gate APIs for subprocesses and observe `cancelled`.
/// No report/baseline write, acceptance or automatic retry occurs.
/// With `Config::select`, the closure must use `config.selection` before rendering
/// and write its exact `selection.json` receipt after complete selected output.
/// This in-process API never mutates the parent's process environment.
pub fn capture_local(
    paths: Paths<'_>,
    config: &Config,
    cancelled: &AtomicBool,
    producer: impl FnMut(&Path) -> Result<()>,
    environment: impl FnMut() -> Result<serde_json::Value>,
) -> Result<Manifest> {
    capture_local_timed(
        paths,
        config,
        cancelled,
        producer,
        environment,
        &mut crate::timing::Timings::default(),
    )
}

/// Local consumer timing; build and host wait can be recorded separately by the caller.
pub fn capture_local_timed(
    paths: Paths<'_>,
    config: &Config,
    cancelled: &AtomicBool,
    producer: impl FnMut(&Path) -> Result<()>,
    environment: impl FnMut() -> Result<serde_json::Value>,
    timings: &mut crate::timing::Timings,
) -> Result<Manifest> {
    ensure!(
        config.environment_command.is_some(),
        "Local capture requires a raw-producer configuration"
    );
    let started = std::time::Instant::now();
    let result = capture_steps(
        paths,
        config,
        cancelled,
        &mut (producer, environment, &mut *timings),
        |callbacks| {
            callbacks
                .2
                .measure("environment_probe_ms", || (callbacks.1)())
                .map(Some)
        },
        |callbacks, run| callbacks.2.measure("producer_ms", || (callbacks.0)(run)),
    );
    timings.record("capture_total_ms", started);
    result
}

/// External capture timing without a new supervisor or altered execution limits.
/// Producer duration includes any consumer build and descendant cleanup; it is not GPU time.
pub fn capture_timed(
    paths: Paths<'_>,
    config: &Config,
    execution: &Execution,
    cancelled: &AtomicBool,
    timings: &mut crate::timing::Timings,
) -> Result<Manifest> {
    let started = std::time::Instant::now();
    let result = capture_inner(
        paths,
        config,
        execution,
        cancelled,
        &mut |command, limits, stdout| {
            timings.measure(
                if stdout {
                    "environment_probe_ms"
                } else {
                    "producer_ms"
                },
                || process::run(command, limits, cancelled, stdout),
            )
        },
    );
    timings.record("capture_total_ms", started);
    result
}

fn capture_inner(
    paths: Paths<'_>,
    config: &Config,
    execution: &Execution,
    cancelled: &AtomicBool,
    execute: &mut impl FnMut(&mut Command, &Limits, bool) -> Result<process::Output>,
) -> Result<Manifest> {
    capture_steps(
        paths,
        config,
        cancelled,
        execute,
        |execute| {
            config
                .environment_command
                .as_ref()
                .map(|args| environment(args, paths.repo, execution, execute))
                .transpose()
        },
        |execute, run| invoke(config, paths.repo, run, execution, execute),
    )
}

fn capture_steps<T>(
    paths: Paths<'_>,
    config: &Config,
    cancelled: &AtomicBool,
    context: &mut T,
    mut probe: impl FnMut(&mut T) -> Result<Option<serde_json::Value>>,
    mut produce: impl FnMut(&mut T, &Path) -> Result<()>,
) -> Result<Manifest> {
    config.validate_schema()?;
    ensure!(!cancelled.load(Ordering::SeqCst), "Capture cancelled");
    let Paths {
        repo,
        run,
        baselines,
        report,
    } = paths;
    let repo = repo.canonicalize()?;
    let run = repo.join(run);
    let baseline = repo.join(baselines);
    let report = repo.join(report);
    store::disjoint(&[&run, &baseline, &report])?;
    ensure!(
        !run.exists(),
        "Run already exists; never overwrite a capture"
    );
    let git_limits = crate::inventory::git_limits();
    let before = store::source_hashes_with(
        &repo,
        &config.source_policy,
        Some(&baseline),
        &git_limits,
        cancelled,
    )?;
    if let Some(selection) = &config.selection {
        if config.environment_command.is_none() {
            return Err(crate::diagnostic::Code::SelectionUnsupported.error("Story selection requires a raw producer; legacy self-sealed captures are unsupported").into());
        }
        selection.check_sources(&repo, &before)?;
    }
    let environment_before = probe(context)?;
    produce(context, &run)?;
    ensure!(
        before
            == store::source_hashes_with(
                &repo,
                &config.source_policy,
                Some(&baseline),
                &git_limits,
                cancelled
            )?,
        "Sources changed during capture"
    );
    if let Some(before_environment) = environment_before {
        let after = probe(context)?.context("Environment probe disappeared during capture")?;
        if before_environment != after {
            return Err(crate::diagnostic::Code::EnvironmentMismatch
                .error("Environment changed during capture")
                .into());
        }
        ensure!(
            !run.join("capture.json").exists(),
            "Raw producer must not seal its own capture"
        );
        let cases: Vec<String> = store::read_json(&run.join("inventory.json"))?;
        let images = crate::parallel::map(&cases, |case| {
            Ok((
                case.clone(),
                store::digest(&store::image_path(&run, case)?)?,
            ))
        })?
        .into_iter()
        .collect();
        // Optional ordered journeys; validated with the complete inventory below.
        let sequences_path = run.join("sequences.json");
        store::no_symlinks(&sequences_path)?;
        let sequences = if sequences_path.exists() {
            store::read_json(&sequences_path)?
        } else {
            Vec::new()
        };
        let manifest = Manifest {
            schema: if config.selection.is_some() { 2 } else { 1 },
            cases,
            environment: before_environment,
            images,
            sources: before.clone(),
            source_policy: config.source_policy.clone(),
            selection: config.selection.clone(),
            sequences,
        };
        if let Some(selection) = &manifest.selection {
            // Explicit receipt prevents an unaware producer from ignoring the
            // request (even if this happens to select every current story).
            let receipt_path = run.join("selection.json");
            store::no_symlinks(&receipt_path)?;
            let receipt: crate::selection::Selection = store::read_json(&receipt_path)
                .context("Producer did not acknowledge story selection")?;
            ensure!(&receipt == selection, "Producer selection receipt differs");
            selection.validate_capture(&manifest)?;
        } else {
            ensure!(
                !run.join("selection.json").exists(),
                "Full capture has an unexpected selection receipt"
            );
        }
        store::validate_images(&run, &manifest)?;
        ensure!(
            !cancelled.load(Ordering::SeqCst),
            "Capture cancelled before sealing"
        );
        store::write_json(&run.join("capture.json"), &manifest)?;
        if let Err(error) = store::load_manifest(&run) {
            fs::remove_file(run.join("capture.json"))?;
            return Err(error);
        }
    }
    let manifest = store::load_manifest(&run)?;
    ensure!(
        manifest.selection == config.selection,
        "Capture selection differs from request"
    );
    ensure!(
        manifest.source_policy.same_inputs(&config.source_policy),
        "Producer source policy does not match configured capture inputs"
    );
    ensure!(
        manifest.sources == before,
        "Producer source inventory does not match configured capture inputs"
    );
    // The CLI formerly reached this check inside compare. A library caller
    // must receive a validated manifest even when it does not generate a report.
    store::validate_images(&run, &manifest)?;
    ensure!(!cancelled.load(Ordering::SeqCst), "Capture cancelled");
    Ok(manifest)
}

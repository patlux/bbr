use anyhow::{ensure, Context, Result};
use bbr_visual::{environment, provenance, runner, Suite};
use cargo_gpui_visual::{
    capture,
    collection::{self, Suite as CollectionSuite},
    config::Config,
    store,
};
use std::{
    path::PathBuf,
    process::ExitCode,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

fn run() -> Result<u8> {
    let cancelled = Arc::new(AtomicBool::new(false));
    for signal in [
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGTERM,
        signal_hook::consts::SIGHUP,
    ] {
        signal_hook::flag::register(signal, cancelled.clone())?;
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    if bbr_visual::guest::dispatch(&args, &cancelled)? {
        return Ok(0);
    }
    let repo = std::fs::canonicalize(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))?;
    if let Some(code) = bbr_visual::read_only::dispatch(&repo, &args)? {
        return Ok(code);
    }
    if let Some(code) = bbr_visual::review::dispatch(&repo, &args, &cancelled)? {
        return Ok(code);
    }
    let command = args.first().context(
        "command required: environment, produce, check, compare, update, doctor, list, inspect",
    )?;
    match command.as_str() {
        "prepare" => {
            ensure!(
                args.len() == 3,
                "prepare requires payload purpose and fresh absolute payload directory"
            );
            let host = bbr_visual::prepare::prepare(
                &repo,
                bbr_visual::payload::Purpose::parse(&args[1])?,
                &PathBuf::from(&args[2]),
                &cancelled,
            )?;
            println!("{}", serde_json::to_string(&host)?);
        }
        "environment" => {
            ensure!(args.len() == 2, "environment requires suite");
            println!(
                "{}",
                environment::probe(&repo, Suite::parse(&args[1])?, &cancelled)?
            );
        }
        "produce" => {
            ensure!(args.len() == 2, "produce requires suite");
            let output = PathBuf::from(
                std::env::var_os("GPUI_VISUAL_OUTPUT").context("capture output required")?,
            );
            runner::produce(&repo, Suite::parse(&args[1])?, &output, &cancelled)?;
        }
        "check" => {
            ensure!(args.len() == 1, "check takes no arguments");
            provenance::verify(&repo)?;
            runner::require_vm_execution()?;
            let stamp = format!(
                "{}-{}",
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
                std::process::id()
            );
            let batch = repo.join("reports/visual/batches").join(stamp);
            store::no_symlinks(&batch)?;
            std::fs::create_dir_all(&batch)?;
            let mut suites = Vec::new();
            for suite in [Suite::Gallery, Suite::Browser] {
                let config = Config::load(&repo.join(suite.config()))?;
                let run = batch.join(suite.id());
                let report = batch.join(format!("{}-diagnostic", suite.id()));
                // One supervisor owns build/native groups. Portable sealing
                // calls local Rust policy without spawning another supervisor.
                if let Err(error) = capture::capture_local(
                    capture::Paths {
                        repo: &repo,
                        run: &run,
                        baselines: &repo.join(&config.baselines),
                        report: &report,
                    },
                    &config,
                    &cancelled,
                    |output| runner::produce(&repo, suite, output, &cancelled),
                    || environment::probe(&repo, suite, &cancelled),
                ) {
                    eprintln!("{} capture failed: {error:#}", suite.id());
                    std::fs::write(
                        batch.join(format!("{}-error.txt", suite.id())),
                        format!("{error:#}\n"),
                    )?;
                }
                ensure!(
                    !cancelled.load(Ordering::SeqCst),
                    "visual check cancelled; partial evidence retained without a review"
                );
                suites.push(CollectionSuite {
                    id: suite.id().into(),
                    label: match suite {
                        Suite::Gallery => "Gallery",
                        Suite::Browser => "Browser chrome",
                    }
                    .into(),
                    repo: repo.clone(),
                    run,
                    baselines: repo.join(config.baselines),
                    catalog: format!("scripts/visual/catalogs/{}.json", suite.id()),
                });
            }
            let report = batch.join("review");
            let result = collection::compare(&suites, &report)?;
            println!(
                "{}: {}",
                if result.passed {
                    "PASS"
                } else {
                    "REVIEW REQUIRED / UNPROVEN COVERAGE"
                },
                report.join("index.html").display()
            );
            return Ok(if result.passed { 0 } else { 1 });
        }
        "update" => {
            ensure!(
                args.len() >= 3,
                "update requires explicit reviewed report and suite:case selections"
            );
            // No latest lookup, adoption, recapture, or acceptance-time build.
            collection::accept(&PathBuf::from(&args[1]), &args[2..], &[])?;
        }
        _ => anyhow::bail!("unknown visual command: {command}"),
    }
    Ok(0)
}
fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("INVALID: {error:#}");
            ExitCode::from(2)
        }
    }
}

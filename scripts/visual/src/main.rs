use anyhow::{ensure, Context, Result};
use bbr_visual::{environment, provenance, runner, vm_check, Suite};
use cargo_gpui_visual::collection;
use std::{
    path::PathBuf,
    process::ExitCode,
    sync::{atomic::AtomicBool, Arc},
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
        "command required: environment, produce, check, compare, update, gate-recover, doctor, list, inspect",
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
            // The official route: explicit suites, each captured in its own
            // disposable Tart guest and sealed by upstream, then one Collection
            // review. No host producer, no acceptance, no implicit latest.
            let suites = vm_check::parse_suites(&args[1..])?;
            provenance::verify(&repo)?;
            let stamp = format!(
                "{}-{}",
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
                std::process::id()
            );
            let outcome = vm_check::check(&repo, &suites, &stamp, &cancelled)?;
            println!(
                "{}: {}",
                if outcome.passed {
                    "PASS"
                } else {
                    "REVIEW REQUIRED / UNPROVEN COVERAGE"
                },
                outcome.report.join("index.html").display()
            );
            println!("batch: {}", outcome.batch.display());
            return Ok(if outcome.passed { 0 } else { 1 });
        }
        "gate-recover" => {
            // The host gate keeps durable unfinished-lease evidence after a
            // failed or killed run. Recovery is explicit: the caller attests
            // the exact inspected lock bytes (sha256) and a reason; upstream
            // proves the recorded owner/groups absent and never deletes the
            // lock or touches captures/baselines.
            ensure!(
                args.len() == 3,
                "gate-recover requires the inspected lock sha256 and a reason"
            );
            let options = runner::options(&repo, &cancelled)?;
            cargo_gpui_visual::gate::Gate::recover(&options, &args[1], &args[2], &cancelled)?;
            println!("host gate recovered: {}", options.path.display());
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

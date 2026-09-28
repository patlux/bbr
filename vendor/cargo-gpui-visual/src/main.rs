use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{
    accept, compare,
    store::{self, Manifest, Policy},
};
use clap::Parser;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

#[derive(Parser)]
#[command(
    version,
    about = "Real GPUI snapshots and exact, local visual review. Never auto-accepts or starts a server."
)]
struct Cli {
    #[arg(value_parser=["check","compare","accept","doctor","sources","validate"])]
    command: String,
    #[arg(long, default_value = ".")]
    repo: PathBuf,
    #[arg(long, default_value = "visual.toml")]
    config: PathBuf,
    #[arg(long)]
    run: Option<PathBuf>,
    #[arg(long)]
    baselines: Option<PathBuf>,
    #[arg(long)]
    report: Option<PathBuf>,
    #[arg(long = "case")]
    cases: Vec<String>,
    #[arg(long)]
    adopt_environment: bool,
    #[arg(long)]
    policy: Option<PathBuf>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    schema: u32,
    capture: Vec<String>,
    /// Optional argv emitting a JSON environment object before and after capture.
    /// Required for raw producers; already-sealed producers own this check themselves.
    environment_command: Option<Vec<String>>,
    baselines: PathBuf,
    report: PathBuf,
    runs: PathBuf,
    source_policy: Policy,
}
fn environment(args: &[String], repo: &Path) -> Result<serde_json::Value> {
    ensure!(!args.is_empty(), "Environment command is empty");
    let output = Command::new(&args[0])
        .args(&args[1..])
        .current_dir(repo)
        .output()?;
    ensure!(output.status.success(), "Environment probe failed");
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn invoke(args: &[String], repo: &Path, run: &Path) -> Result<()> {
    ensure!(!args.is_empty(), "Capture command is empty");
    let replaced: Vec<_> = args
        .iter()
        .map(|a| a.replace("{run}", &run.to_string_lossy()))
        .collect();
    let status = Command::new(&replaced[0])
        .args(&replaced[1..])
        .current_dir(repo)
        .env("GPUI_VISUAL_OUTPUT", run)
        .status()?;
    ensure!(status.success(), "Capture failed: {status}");
    Ok(())
}
fn run(cli: Cli) -> Result<u8> {
    let repo = cli.repo.canonicalize()?;
    if cli.command == "sources" {
        let policy: Policy =
            store::read_json(&repo.join(cli.policy.context("--policy required")?))?;
        println!(
            "{}",
            serde_json::to_string(&store::source_hashes(&repo, &policy)?)?
        );
        return Ok(0);
    }
    if cli.command == "validate" {
        let run = repo.join(cli.run.context("--run required")?);
        let m = store::load_manifest(&run)?;
        store::check_sources(&repo, &m)?;
        store::validate_images(&run, &m)?;
        return Ok(0);
    }
    let config_path = repo.join(&cli.config);
    let config: Option<Config> = if config_path.is_file() {
        Some(toml::from_str(&fs::read_to_string(&config_path)?)?)
    } else {
        None
    };
    if let Some(c) = &config {
        ensure!(c.schema == 1, "Unsupported config schema");
    }
    if cli.command == "doctor" {
        let c = config.context("visual.toml required")?;
        ensure!(!c.capture.is_empty(), "Missing capture command");
        let sources = store::source_hashes(&repo, &c.source_policy)?;
        ensure!(!sources.is_empty(), "No capture sources");
        println!("Configuration valid; {} source inputs. Capture command: {:?}. No build/device/server started.",sources.len(),c.capture);
        return Ok(0);
    }
    ensure!(
        cli.command == "accept" || (cli.cases.is_empty() && !cli.adopt_environment),
        "--case/--adopt-environment are acceptance-only"
    );
    let baseline = cli
        .baselines
        .or_else(|| config.as_ref().map(|c| c.baselines.clone()))
        .context("--baselines required")?;
    let report = cli
        .report
        .or_else(|| config.as_ref().map(|c| c.report.clone()))
        .context("--report required")?;
    let baseline = repo.join(baseline);
    let report = repo.join(report);
    let run = if let Some(run) = cli.run {
        repo.join(run)
    } else if cli.command == "check" {
        let c = config.as_ref().context("visual.toml required for check")?;
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        repo.join(&c.runs)
            .join(format!("{stamp}-{}", std::process::id()))
    } else {
        anyhow::bail!("--run required; acceptance never chooses an implicit latest capture")
    };
    if cli.command == "check" {
        let c = config.as_ref().context("visual.toml required for check")?;
        store::disjoint(&[&run, &baseline, &report])?;
        ensure!(
            !run.exists(),
            "Run already exists; never overwrite a capture"
        );
        let before = store::source_hashes(&repo, &c.source_policy)?;
        let environment_before = c
            .environment_command
            .as_ref()
            .map(|args| environment(args, &repo))
            .transpose()?;
        invoke(&c.capture, &repo, &run)?;
        ensure!(
            before == store::source_hashes(&repo, &c.source_policy)?,
            "Sources changed during capture"
        );
        if let Some(before_environment) = environment_before {
            let after = environment(c.environment_command.as_ref().unwrap(), &repo)?;
            ensure!(
                before_environment == after,
                "Environment changed during capture"
            );
            ensure!(
                !run.join("capture.json").exists(),
                "Raw producer must not seal its own capture"
            );
            let cases: Vec<String> = store::read_json(&run.join("inventory.json"))?;
            let images = cases
                .iter()
                .map(|case| {
                    Ok((
                        case.clone(),
                        store::digest(&store::image_path(&run, case)?)?,
                    ))
                })
                .collect::<Result<_>>()?;
            let manifest = Manifest {
                schema: 1,
                cases,
                environment: before_environment,
                images,
                sources: before.clone(),
                source_policy: c.source_policy.clone(),
            };
            store::validate_images(&run, &manifest)?;
            store::write_json(&run.join("capture.json"), &manifest)?;
            if let Err(error) = store::load_manifest(&run) {
                fs::remove_file(run.join("capture.json"))?;
                return Err(error);
            }
        }
        // Otherwise producers must have sealed their own before/after environment.
        let m: Manifest = store::load_manifest(&run)?;
        ensure!(
            m.source_policy.same_inputs(&c.source_policy),
            "Producer source policy does not match configured capture inputs"
        );
        ensure!(
            m.sources == before,
            "Producer source inventory does not match configured capture inputs"
        );
    }
    if cli.command == "accept" {
        accept::accept(
            &repo,
            &run,
            &baseline,
            &report,
            &cli.cases,
            cli.adopt_environment,
        )?;
        println!(
            "Accepted {} explicitly selected references. Inspect git diff; no commit created.",
            cli.cases.len()
        );
        return Ok(0);
    }
    let result = compare::compare(&repo, &run, &baseline, &report)?;
    let same = result.results.iter().filter(|r| r.status == "same").count();
    println!(
        "{}: {same}/{} exact · {} errors{}",
        if result.passed { "PASS" } else { "FAIL" },
        result.results.len(),
        result.errors.len(),
        if result.environment_mismatch.is_some() {
            " · environment mismatch"
        } else {
            ""
        }
    );
    for i in result.results.iter().filter(|i| i.status != "same") {
        println!(
            "  {}: {} · {} pixels",
            i.case,
            i.status,
            i.changed_pixels
                .map_or_else(|| "–".into(), |v| v.to_string())
        );
    }
    for error in &result.errors {
        eprintln!("  {error}");
    }
    println!(
        "Report: {}\nRun: {}",
        report.join("index.html").display(),
        run.display()
    );
    // Stable machine-readable verdict is in review.json; exit 2 is reserved for invalid input.
    Ok(if result.passed { 0 } else { 1 })
}
fn main() -> ExitCode {
    let mut args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_some_and(|s| s == "gpui-visual") {
        args.remove(1);
    }
    match run(Cli::parse_from(args)) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("INVALID: {error:#}");
            ExitCode::from(2)
        }
    }
}

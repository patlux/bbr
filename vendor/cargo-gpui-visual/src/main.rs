mod cli;
mod cli_output;
mod interrupt;

use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{
    accept,
    agent::{self, Response},
    capture, collection, compare,
    config::Config,
    store::{self, Policy},
};
use clap::Parser;
use cli::{Cli, Request};
use serde_json::json;
use std::{io::IsTerminal, process::ExitCode};

fn run(
    cli: Request,
    timings: &mut cargo_gpui_visual::timing::Timings,
    failure_context: &mut serde_json::Value,
) -> Result<Response> {
    let reply = |code, data| Response::new(cli.command, code, data);
    if cli.command == "skill" {
        let content = cargo_gpui_visual::skill::get(cli.reference.as_deref())?;
        if !cli.json {
            print!("{content}");
        }
        return Ok(reply(
            0,
            json!({"name":"gpui-visual","reference":cli.reference,"content":content}),
        ));
    }
    if cli.command == "compact-baselines" {
        let root = store::absolute(
            &cli.repo
                .join(cli.baselines.context("--baselines required")?),
        )?;
        let result = cargo_gpui_visual::baseline::compact(
            &root,
            &cli.expected_hash.context("--expected-hash required")?,
            cli.dry_run,
        )?;
        if !cli.json {
            println!("{} baseline metadata: {} → {} bytes, {} cases / {} captures. No images, environment or timelines accepted; regenerate existing reviews before acceptance.",
                if result.written { "Compacted" } else { "Inspected" }, result.old_bytes, result.new_bytes, result.cases, result.captures);
        }
        return Ok(reply(
            0,
            json!({"baselines":root,"compaction":result,"accepted":[],"next_step":"Regenerate reports after migration; prior baseline-state seals are stale."}),
        ));
    }
    if cli.command == "convert-baselines" {
        let root = store::absolute(
            &cli.repo
                .join(cli.baselines.context("--baselines required")?),
        )?;
        let format = cargo_gpui_visual::codec::Format::parse(
            cli.format.as_deref().context("--format required")?,
        )?;
        let result = cargo_gpui_visual::baseline::convert(
            &root,
            format,
            &cli.expected_hash.context("--expected-hash required")?,
            cli.dry_run,
        )?;
        if !cli.json {
            println!("{} {} references {} → {}: images {} → {} bytes, metadata {} → {} bytes. Every image decoded and proven pixel-identical; nothing accepted; regenerate existing reviews before acceptance.",
                if result.written { "Converted" } else { "Verified" }, result.cases,
                result.from.extension(), result.to.extension(),
                result.old_image_bytes, result.new_image_bytes,
                result.old_metadata_bytes, result.new_metadata_bytes);
        }
        return Ok(reply(
            0,
            json!({"baselines":root,"conversion":result,"accepted":[],"next_step":"Regenerate reports after conversion; prior baseline-state seals are stale."}),
        ));
    }
    if cli.command == "inspect" {
        let root = store::absolute(&cli.repo.join(cli.report.context("--report required")?))?;
        let data = cargo_gpui_visual::inspect::inspect(&root, &cli.inspect)?;
        if !cli.json {
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        return Ok(reply(0, data));
    }
    if matches!(cli.command, "accept-collection" | "review-collection") {
        ensure!(
            cli.repo == std::path::Path::new(".")
                && cli.config == std::path::Path::new("visual.toml"),
            "Collection commands use only explicit suite authorities, not --repo/--config"
        );
        let root = store::absolute(&cli.report.context("--report required")?)?;
        if cli.command == "accept-collection" {
            collection::accept(&root, &cli.cases, &cli.adopt_suites)?;
            if !cli.json {
                println!("Accepted {} explicit collection cases. Inspect reference diffs and acceptance.json; no commit created.", cli.cases.len());
            }
            return Ok(reply(
                0,
                json!({"accepted":cli.cases,"adopted_suites":cli.adopt_suites,"journal":root.join("acceptance.json")}),
            ));
        }
        let input = collection::Input::load(&cli.collection.context("--collection required")?)?;
        let result = collection::compare(&input.suites, &root)?;
        if !cli.json {
            println!(
                "{}: {} suites. Report: {}",
                if result.passed { "PASS" } else { "FAIL" },
                result.suites.len(),
                root.join("index.html").display()
            );
        }
        let suites: Vec<_> = result.suites.iter().map(|s| json!({"id":s.suite.id,"passed":s.passed,"errors":s.errors.len(),"review":s.review.as_ref().map(agent::review_summary)})).collect();
        return Ok(reply(
            if result.passed { 0 } else { 1 },
            json!({"passed":result.passed,"suites":suites,"report":root,"collection":root.join("collection.json")}),
        ));
    }
    let repo = cli.repo.canonicalize()?;
    let config_path = repo.join(&cli.config);
    let mut config = if config_path.is_file() {
        Some(Config::load(&config_path)?)
    } else {
        None
    };
    if cli.command == "list" {
        let c = config.as_ref().context("visual.toml required for list")?;
        let (path, catalog) = cargo_gpui_visual::selection::Catalog::load(&repo, c)?;
        let sources =
            store::source_hashes(&repo, &c.source_policy, Some(&repo.join(&c.baselines)))?;
        ensure!(
            sources.get(&path) == Some(&store::digest(&repo.join(path))?),
            "Story catalog must be a sealed source input"
        );
        if !cli.json {
            println!("{}", serde_json::to_string_pretty(&catalog)?);
        }
        return Ok(reply(0, serde_json::to_value(catalog)?));
    }
    if !cli.stories.is_empty() {
        config = Some(
            config
                .as_ref()
                .context("visual.toml required for story selection")?
                .select(&repo, &cli.stories)?,
        );
    }
    let baseline = cli
        .baselines
        .or_else(|| config.as_ref().map(|c| c.baselines.clone()))
        .map(|p| repo.join(p));
    if cli.command == "sources" {
        let policy: Policy =
            store::read_json(&repo.join(cli.policy.context("--policy required")?))?;
        let sources = store::source_hashes(&repo, &policy, baseline.as_deref())?;
        if !cli.json {
            println!("{}", serde_json::to_string(&sources)?);
        }
        return Ok(reply(0, serde_json::to_value(sources)?));
    }
    if cli.command == "validate" {
        let run = repo.join(cli.run.context("--run required")?);
        let m = store::load_manifest(&run)?;
        store::check_sources(&repo, &m, baseline.as_deref())?;
        store::validate_images(&run, &m)?;
        return Ok(reply(
            0,
            json!({"run":run,"sources_and_images_valid":true,"baseline_environment_checked":false,"producer_executed":false}),
        ));
    }
    if cli.command == "doctor" {
        let c = config.context("visual.toml required")?;
        ensure!(!c.capture.is_empty(), "Missing capture command");
        let sources = store::source_hashes(&repo, &c.source_policy, baseline.as_deref())?;
        ensure!(!sources.is_empty(), "No capture sources");
        if !cli.json {
            println!("Configuration valid; {} source inputs. Capture command: {:?}. No build/device/server started.", sources.len(), c.capture);
        }
        return Ok(reply(
            0,
            json!({"source_inputs":sources.len(),"capture_argv":c.capture,"producer_executed":false,"environment_probed":false,"story_catalog_configured":c.story_catalog.is_some(),"producer_selection_verified":false,"native_authorization":"consumer_owned","host_gate_checked":false}),
        ));
    }
    let baseline = baseline.context("--baselines required")?;
    if cli.command == "compare"
        && store::load_manifest(&repo.join(cli.run.as_ref().context("--run required")?))?
            .selection
            .is_some()
    {
        ensure!(
            cli.report.is_some(),
            "Selected-story comparison requires --report; preserve the full-suite report"
        );
    }
    let report = repo.join(
        cli.report
            .or_else(|| config.as_ref().map(|c| c.report.clone()))
            .context("--report required")?,
    );
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
    *failure_context = json!({"artifacts":{"repo":repo,"run":run,"baselines":baseline,"report":report},
        "warning":"Attempt paths only; files or valid seals may not exist. Inspect before retrying; never overwrite a failed run."});
    if cli.command == "check" {
        let c = config.as_ref().context("visual.toml required for check")?;
        if cli.dry_run {
            let plan = cargo_gpui_visual::plan::check(
                capture::Paths {
                    repo: &repo,
                    run: &run,
                    baselines: &baseline,
                    report: &report,
                },
                c,
                cli.host_gate,
                &cli.compatibility_gate,
            )?;
            if !cli.json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            }
            return Ok(reply(0, serde_json::to_value(plan)?));
        }
        let interrupts = interrupt::Interrupts::install()?;
        if cli.host_gate {
            #[cfg(unix)]
            {
                let mut options = cargo_gpui_visual::gate::Options::host()?;
                options.compatibility = cli.compatibility_gate;
                capture::capture_guarded_timed(
                    capture::Paths {
                        repo: &repo,
                        run: &run,
                        baselines: &baseline,
                        report: &report,
                    },
                    c,
                    &capture::Execution::default(),
                    &interrupts.cancelled,
                    &options,
                    timings,
                )?;
            }
            #[cfg(not(unix))]
            anyhow::bail!("Host gate is not implemented for this platform");
        } else {
            capture::capture_timed(
                capture::Paths {
                    repo: &repo,
                    run: &run,
                    baselines: &baseline,
                    report: &report,
                },
                c,
                &capture::Execution::default(),
                &interrupts.cancelled,
                timings,
            )?;
        }
    }
    if cli.command == "accept" {
        // clap counts a bool flag's default as present, so `requires` cannot guard this.
        ensure!(cli.all || !cli.yes, "--yes only confirms --all");
        let format =
            accept::store_format(&baseline, config.as_ref().and_then(|c| c.baseline_format))?;
        let accepted = if cli.all {
            let reviewed = accept::preview_all(&run, &report)?;
            accept::confirm_all(&reviewed, cli.yes)?;
            accept::accept_all_with_format(
                &repo,
                &run,
                &baseline,
                &report,
                &reviewed,
                cli.adopt_environment,
                format,
            )?;
            reviewed.ids()
        } else {
            accept::accept_with_format(
                &repo,
                &run,
                &baseline,
                &report,
                &cli.cases,
                cli.adopt_environment,
                format,
            )?;
            cli.cases.clone()
        };
        if !cli.json {
            println!(
                "Accepted {} {} references. Inspect git diff; no commit created.",
                accepted.len(),
                if cli.all {
                    "confirmed reviewed"
                } else {
                    "explicitly selected"
                }
            );
        }
        return Ok(reply(
            0,
            json!({"accepted":accepted,"all":cli.all,"adopted_environment":cli.adopt_environment,"baselines":baseline,"next_step":"Inspect reference diffs; no commit created."}),
        ));
    }
    let options = config.map(|c| c.review).unwrap_or_default();
    let result = compare::compare_timed(&repo, &run, &baseline, &report, &options, timings)?;
    if !cli.json {
        let color = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
        println!(
            "{}",
            cargo_gpui_visual::terminal::summary(
                &result,
                cargo_gpui_visual::terminal::Paths {
                    repo: &repo,
                    run: &run,
                    baselines: &baseline,
                    report: &report
                },
                color
            )
        );
    }
    Ok(reply(
        if result.passed { 0 } else { 1 },
        agent::review_result(&result, &repo, &run, &baseline, &report),
    ))
}
fn main() -> ExitCode {
    let mut args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_some_and(|s| s == "gpui-visual") {
        args.remove(1);
    }
    let json_requested = args
        .iter()
        .take_while(|a| *a != "--")
        .any(|a| a == "--json");
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            if !json_requested
                || matches!(
                    error.kind(),
                    clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
                )
            {
                error.exit();
            }
            let cause = cargo_gpui_visual::diagnostic::Code::InvalidArguments
                .error(error.to_string())
                .into();
            println!(
                "{}",
                serde_json::to_string(&Response::failure("parse", &cause))
                    .expect("serializable response")
            );
            return ExitCode::from(2);
        }
    };
    let request = Request::from(cli);
    let command = request.command;
    let output = if request.json {
        match cli_output::JsonOutput::begin() {
            Ok(output) => Some(output),
            Err(e) => {
                eprintln!("INVALID: JSON stdout isolation failed: {e:#}");
                return ExitCode::from(2);
            }
        }
    } else {
        None
    };
    let mut timings = cargo_gpui_visual::timing::Timings::default();
    let started = std::time::Instant::now();
    let mut failure_context = serde_json::Value::Null;
    let mut result = match run(request, &mut timings, &mut failure_context) {
        Ok(result) => result,
        Err(error) => {
            if output.is_none() {
                eprintln!("INVALID: {error:#}");
            }
            let mut response = Response::failure(command, &error);
            response.data = failure_context;
            response
        }
    };
    timings.record("total_ms", started);
    result.timings_ms = timings;
    if let Some(output) = output {
        if let Err(e) = output.publish(&result) {
            eprintln!("INVALID: JSON output: {e:#}");
            return ExitCode::from(2);
        }
    }
    ExitCode::from(result.exit_code)
}

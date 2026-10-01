//! Strict read-only CLI surface; structured failures use the upstream envelope.
use crate::{discovery, Suite};
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{agent::Response, inspect};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

fn value(args: &[String], index: &mut usize) -> Result<String> {
    *index += 1;
    let value = args.get(*index).context("option requires a value")?;
    ensure!(!value.starts_with("--"), "option requires a value");
    Ok(value.clone())
}

fn execute(repo: &Path, args: &[String]) -> Result<Value> {
    let mut suite = None;
    let mut run = None;
    let mut report = None;
    let mut stories = Vec::new();
    let mut options = inspect::Options::default();
    let mut flags = BTreeSet::new();
    let mut index = 1;
    let command = &args[0];
    while index < args.len() {
        let flag = args[index].as_str();
        ensure!(
            flag == "--story" || flags.insert(flag),
            "duplicate option: {flag}"
        );
        match (command.as_str(), flag) {
            (_, "--json") | ("check", "--dry-run") => {}
            ("list" | "check", "--suite") => suite = Some(Suite::parse(&value(args, &mut index)?)?),
            ("check", "--run") => run = Some(PathBuf::from(value(args, &mut index)?)),
            ("check" | "inspect", "--report") => {
                report = Some(PathBuf::from(value(args, &mut index)?))
            }
            ("check", "--story") => stories.push(value(args, &mut index)?),
            ("inspect", "--case") => options.case = Some(value(args, &mut index)?),
            ("inspect", "--tick") => options.tick = Some(value(args, &mut index)?.parse()?),
            ("inspect", "--limit") => options.limit = value(args, &mut index)?.parse()?,
            ("inspect", "--offset") => options.offset = value(args, &mut index)?.parse()?,
            _ => anyhow::bail!("unknown option for {command}: {flag}"),
        }
        index += 1;
    }
    match command.as_str() {
        "doctor" => discovery::doctor(repo),
        "list" => discovery::list(repo, suite.context("--suite required")?),
        "check" => {
            ensure!(
                flags.contains("--dry-run"),
                "read-only check requires --dry-run"
            );
            discovery::preview(
                repo,
                suite.context("--suite required")?,
                &run.context("--run required for a deterministic preview")?,
                &report.context("--report required; no implicit latest")?,
                &stories,
            )
        }
        "inspect" => inspect::inspect(
            &repo.join(report.context("--report required; no implicit latest")?),
            &options,
        ),
        _ => anyhow::bail!("unknown read-only command"),
    }
}

/// None means an existing mutating/native route, which has its own authorization
/// guard. Do not accidentally turn malformed dry-runs into executable checks.
pub fn dispatch(repo: &Path, args: &[String]) -> Result<Option<u8>> {
    let Some(command) = args.first() else {
        return Ok(None);
    };
    if !matches!(command.as_str(), "doctor" | "list" | "inspect")
        && !(command == "check" && args.iter().any(|a| a == "--dry-run"))
    {
        return Ok(None);
    }
    let json = args.iter().any(|a| a == "--json");
    let result = execute(repo, args);
    let response = match result {
        Ok(data) => Response::new(command, 0, data),
        Err(error) if json => Response::failure(command, &error),
        Err(error) => return Err(error),
    };
    if json {
        println!("{}", serde_json::to_string(&response)?);
    } else if command == "doctor" {
        for suite in response.data["suites"]
            .as_array()
            .context("doctor suites")?
        {
            println!(
                "{}: {} sealed source inputs",
                suite["id"].as_str().unwrap_or("?"),
                suite["source_inputs"]
            );
        }
        println!(
            "Capture blocked pending Tart; current coverage and guest environment are unverified."
        );
    } else {
        println!("{}", serde_json::to_string_pretty(&response.data)?);
    }
    Ok(Some(response.exit_code))
}

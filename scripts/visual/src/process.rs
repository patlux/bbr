//! Use the portable process authority; BBR owns only argv/build interpretation.
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::process::{self, Limits};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
    time::Duration,
};

pub fn output(command: &mut Command, cancelled: &AtomicBool) -> Result<String> {
    let result = process::run(
        command,
        &Limits {
            timeout: Duration::from_secs(30),
            ..Limits::default()
        },
        cancelled,
        true,
    )?;
    ensure!(result.status.success(), "command failed: {command:?}");
    Ok(String::from_utf8(result.stdout)?.trim().to_owned())
}

pub fn build(repo: &Path, arguments: &[&str], cancelled: &AtomicBool) -> Result<Vec<Value>> {
    build_output(
        Command::new("mbx")
            .current_dir(repo)
            .args([
                "build",
                "--locked",
                "--offline",
                "--release",
                "--config",
                "profile.release.build-override.strip=false",
                "--message-format=json-render-diagnostics",
            ])
            .args(arguments),
        cancelled,
    )
}

fn gallery_semantic_command(repo: &Path) -> Command {
    let mut command = Command::new("mbx");
    command.current_dir(repo).args([
        "test",
        "--no-run",
        "--locked",
        "--offline",
        "--release",
        "--config",
        "profile.release.build-override.strip=false",
        "--message-format=json-render-diagnostics",
        "-p",
        "bbr-gallery",
        "--features",
        "visual-capture",
        "--bin",
        "bbr-gallery",
    ]);
    command
}

pub fn build_gallery_semantic(repo: &Path, cancelled: &AtomicBool) -> Result<Vec<Value>> {
    build_output(&mut gallery_semantic_command(repo), cancelled)
}

fn build_output(command: &mut Command, cancelled: &AtomicBool) -> Result<Vec<Value>> {
    let result = process::run(
        command,
        &Limits {
            timeout: Duration::from_secs(900),
            max_output_bytes: 64 * 1024 * 1024,
            ..Limits::default()
        },
        cancelled,
        true,
    )?;
    ensure!(result.status.success(), "BBR producer build failed");
    Ok(String::from_utf8(result.stdout)?
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|value| value["reason"] == "compiler-artifact")
        .collect())
}

pub fn executable(artifacts: &[Value], name: &str, test: bool) -> Result<PathBuf> {
    let matches: Vec<_> = artifacts
        .iter()
        .filter(|a| a["target"]["name"] == name && a["profile"]["test"] == test)
        .filter_map(|a| a["executable"].as_str().map(PathBuf::from))
        .collect();
    ensure!(
        matches.len() == 1,
        "exactly one executable required: {name}"
    );
    let path = matches.into_iter().next().context("missing executable")?;
    ensure!(
        path.is_file(),
        "built executable missing: {}",
        path.display()
    );
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_build_is_feature_enabled_release_no_run_only() {
        let command = gallery_semantic_command(Path::new("/compiled/repo"));
        assert_eq!(command.get_program(), "mbx");
        assert_eq!(command.get_current_dir(), Some(Path::new("/compiled/repo")));
        let args: Vec<_> = command.get_args().map(|a| a.to_str().unwrap()).collect();
        assert_eq!(
            args,
            [
                "test",
                "--no-run",
                "--locked",
                "--offline",
                "--release",
                "--config",
                "profile.release.build-override.strip=false",
                "--message-format=json-render-diagnostics",
                "-p",
                "bbr-gallery",
                "--features",
                "visual-capture",
                "--bin",
                "bbr-gallery"
            ]
        );
        assert_eq!(command.get_envs().count(), 0);
    }
}

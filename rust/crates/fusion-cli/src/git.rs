use anyhow::{bail, Context, Result};
use std::io::{self, Write};
use std::process::{Command, Output};

fn run_git(args: &[&str]) -> Result<Output> {
    Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))
}

fn run_git_checked(args: &[&str]) -> Result<Output> {
    let output = run_git(args)?;
    if output.status.success() {
        return Ok(output);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let message = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("git {} failed with {}", args.join(" "), output.status)
    };
    bail!("{message}");
}

fn ensure_git_repo() -> Result<()> {
    run_git_checked(&["rev-parse", "--is-inside-work-tree"]).map(|_| ())
}

fn current_branch() -> Result<String> {
    let output = run_git_checked(&["branch", "--show-current"])?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn write_stdout(output: &Output) -> Result<()> {
    io::stdout()
        .write_all(&output.stdout)
        .context("failed to write git stdout")?;
    Ok(())
}

pub(crate) fn cmd_status() -> Result<()> {
    ensure_git_repo()?;
    println!("=== Fusion Git Status ===");
    println!("Current branch: {}", current_branch()?);
    println!();
    cmd_changes()
}

pub(crate) fn cmd_create_branch(goal_slug: &str) -> Result<()> {
    ensure_git_repo()?;

    let branch_name = format!("fusion/{goal_slug}");
    let exists = Command::new("git")
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch_name}"),
        ])
        .status()
        .context("failed to check branch existence")?;

    if exists.success() {
        run_git_checked(&["checkout", &branch_name])?;
    } else {
        run_git_checked(&["checkout", "-b", &branch_name])?;
    }

    println!("{branch_name}");
    Ok(())
}

pub(crate) fn cmd_commit(message: &str) -> Result<()> {
    ensure_git_repo()?;

    let status_output = run_git_checked(&["status", "--short"])?;
    if status_output.stdout.is_empty() {
        return Ok(());
    }

    run_git_checked(&["add", "-A"])?;
    run_git_checked(&["commit", "-m", message])?;
    println!("{}", current_branch_commit()?);
    Ok(())
}

fn current_branch_commit() -> Result<String> {
    let output = run_git_checked(&["rev-parse", "--short", "HEAD"])?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub(crate) fn cmd_branch() -> Result<()> {
    ensure_git_repo()?;
    println!("{}", current_branch()?);
    Ok(())
}

pub(crate) fn cmd_changes() -> Result<()> {
    ensure_git_repo()?;

    println!("=== Git Status ===");
    let status_output = run_git_checked(&["status", "--short"])?;
    write_stdout(&status_output)?;
    println!();
    println!("=== Changed Files ===");

    let unstaged = run_git_checked(&["diff", "--name-only"])?;
    write_stdout(&unstaged)?;

    let staged = run_git_checked(&["diff", "--staged", "--name-only"])?;
    write_stdout(&staged)?;
    Ok(())
}

pub(crate) fn cmd_diff() -> Result<()> {
    ensure_git_repo()?;
    let output = run_git_checked(&["diff", "HEAD"])?;
    write_stdout(&output)
}

pub(crate) fn cmd_cleanup(original_branch: Option<&str>) -> Result<()> {
    ensure_git_repo()?;
    if let Some(branch) = original_branch.filter(|value| !value.is_empty()) {
        let exists = Command::new("git")
            .args([
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/heads/{branch}"),
            ])
            .status()
            .context("failed to inspect original branch")?;
        if exists.success() {
            run_git_checked(&["checkout", branch])?;
        } else {
            run_git_checked(&["checkout", "--orphan", branch])?;
        }
    }
    Ok(())
}

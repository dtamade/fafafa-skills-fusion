use anyhow::{Context, Result};
use regex::Regex;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapperResolution {
    pub bin: PathBuf,
    pub attempted: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendRunResult {
    pub output: String,
    pub exit_code: i32,
}

pub fn session_key_for_backend(backend: &str) -> &'static str {
    match backend {
        "claude" => "claude_session",
        _ => "codex_session",
    }
}

pub fn extract_session_id(text: &str) -> Option<String> {
    let re = Regex::new(r"[0-9]{6,}[A-Za-z0-9_-]*").expect("regex compile");
    re.find(text).map(|m| m.as_str().to_string())
}

pub fn resolve_wrapper_bin(explicit: Option<&str>, cwd: &Path) -> Result<WrapperResolution> {
    let mut attempted: Vec<String> = Vec::new();

    if let Some(explicit_bin) = explicit {
        attempted.push(explicit_bin.to_string());
        let explicit_path = PathBuf::from(explicit_bin);
        if is_executable(&explicit_path) {
            return Ok(WrapperResolution {
                bin: explicit_path,
                attempted,
            });
        }
    }

    attempted.push("codeagent-wrapper in PATH".to_string());
    if let Ok(path) = which::which("codeagent-wrapper") {
        return Ok(WrapperResolution {
            bin: path,
            attempted,
        });
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    candidates.push(cwd.join("node_modules/.bin/codeagent-wrapper"));

    if let Ok(home) = env::var("HOME") {
        candidates.push(PathBuf::from(&home).join(".local/bin/codeagent-wrapper"));
        candidates.push(PathBuf::from(home).join(".npm-global/bin/codeagent-wrapper"));
    }

    for candidate in candidates {
        attempted.push(candidate.to_string_lossy().to_string());
        if is_executable(&candidate) {
            return Ok(WrapperResolution {
                bin: candidate,
                attempted,
            });
        }
    }

    anyhow::bail!("missing codeagent-wrapper")
}

pub fn run_backend(
    wrapper_bin: &Path,
    backend: &str,
    prompt: &str,
    session_id: Option<&str>,
    workdir: &Path,
) -> Result<BackendRunResult> {
    let mut cmd = Command::new(wrapper_bin);
    cmd.arg("--backend").arg(backend);

    if let Some(sid) = session_id {
        if !sid.is_empty() {
            cmd.arg("resume").arg(sid).arg("-").arg(workdir);
        } else {
            cmd.arg("-").arg(workdir);
        }
    } else {
        cmd.arg("-").arg(workdir);
    }

    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed launching wrapper: {}", wrapper_bin.display()))?;

    if let Some(mut stdin) = child.stdin.take() {
        if let Err(error) = stdin.write_all(prompt.as_bytes()) {
            if error.kind() != std::io::ErrorKind::BrokenPipe {
                return Err(error).context("failed writing prompt to wrapper stdin");
            }
        }
    }

    let output = child
        .wait_with_output()
        .context("failed waiting wrapper process")?;

    let mut merged = String::new();
    merged.push_str(&String::from_utf8_lossy(&output.stdout));
    merged.push_str(&String::from_utf8_lossy(&output.stderr));

    Ok(BackendRunResult {
        output: merged,
        exit_code: output.status.code().unwrap_or(1),
    })
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    meta.is_file() && (meta.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_session_key_for_backend() {
        assert_eq!(session_key_for_backend("codex"), "codex_session");
        assert_eq!(session_key_for_backend("claude"), "claude_session");
    }

    #[test]
    fn test_extract_session_id() {
        let text = "mock backend:codex\nSESSION_ID: 123456\n";
        assert_eq!(extract_session_id(text).as_deref(), Some("123456"));
    }

    #[test]
    fn test_resolve_wrapper_from_env_path() {
        let dir = tempdir().expect("tempdir");
        let bin = dir.path().join("wrapper");
        fs::write(&bin, "#!/bin/bash\necho ok\n").expect("write wrapper");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&bin).expect("meta").permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&bin, perms).expect("chmod");
        }

        let result =
            resolve_wrapper_bin(Some(&bin.to_string_lossy()), dir.path()).expect("resolve");
        assert_eq!(result.bin, bin);
    }

    #[test]
    fn test_run_backend_mock() {
        let dir = tempdir().expect("tempdir");
        let wrapper = dir.path().join("codeagent-wrapper");
        fs::write(
            &wrapper,
            "#!/bin/bash\necho \"mock backend:$2\"\necho \"SESSION_ID: 123456\"\n",
        )
        .expect("write wrapper");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&wrapper).expect("meta").permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&wrapper, perms).expect("chmod");
        }

        let result = run_backend(&wrapper, "codex", "hello", None, dir.path()).expect("run");
        assert_eq!(result.exit_code, 0);
        assert!(result.output.contains("SESSION_ID: 123456"));
    }
}

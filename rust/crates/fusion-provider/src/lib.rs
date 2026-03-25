use anyhow::{Context, Result};
use regex::Regex;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

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

pub fn session_key_for_backend_role(backend: &str, role: Option<&str>) -> String {
    let normalized_backend = match backend {
        "claude" => "claude",
        _ => "codex",
    };

    match role.and_then(normalize_role) {
        Some(role_name) => format!("{role_name}_{normalized_backend}_session"),
        None => session_key_for_backend(normalized_backend).to_string(),
    }
}

fn normalize_role(role: &str) -> Option<&'static str> {
    match role.trim().to_ascii_lowercase().as_str() {
        "planner" => Some("planner"),
        "coder" => Some("coder"),
        "reviewer" => Some("reviewer"),
        _ => None,
    }
}

pub fn extract_session_id(text: &str) -> Option<String> {
    let re = Regex::new(r"(?m)^SESSION_ID:\s*([^\s]+)").expect("regex compile");
    re.captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
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
    timeout_sec: Option<u64>,
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

    let stdout = child
        .stdout
        .take()
        .context("failed capturing wrapper stdout")?;
    let stderr = child
        .stderr
        .take()
        .context("failed capturing wrapper stderr")?;

    let stdout_handle = thread::spawn(move || {
        let mut buf = Vec::new();
        let mut reader = stdout;
        let _ = reader.read_to_end(&mut buf);
        buf
    });
    let stderr_handle = thread::spawn(move || {
        let mut buf = Vec::new();
        let mut reader = stderr;
        let _ = reader.read_to_end(&mut buf);
        buf
    });

    let deadline = timeout_sec.map(|sec| Instant::now() + Duration::from_secs(sec));
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait().context("failed polling wrapper process")? {
            break status;
        }

        if deadline.is_some_and(|limit| Instant::now() >= limit) {
            timed_out = true;
            let _ = child.kill();
            break child
                .wait()
                .context("failed waiting timed out wrapper process")?;
        }

        thread::sleep(Duration::from_millis(50));
    };

    let stdout_bytes = stdout_handle.join().unwrap_or_default();
    let stderr_bytes = stderr_handle.join().unwrap_or_default();

    let mut merged = String::new();
    merged.push_str(&String::from_utf8_lossy(&stdout_bytes));
    merged.push_str(&String::from_utf8_lossy(&stderr_bytes));

    if timed_out {
        if !merged.is_empty() && !merged.ends_with('\n') {
            merged.push('\n');
        }
        if let Some(sec) = timeout_sec {
            merged.push_str(&format!(
                "[fusion] backend {backend} timed out after {sec}s\n"
            ));
        }
    }

    Ok(BackendRunResult {
        output: merged,
        exit_code: if timed_out {
            124
        } else {
            status.code().unwrap_or(1)
        },
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
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn write_mock_executable(
        dir: &Path,
        base_name: &str,
        unix_body: &str,
        _windows_body: &str,
    ) -> PathBuf {
        #[cfg(windows)]
        let path = dir.join(format!("{base_name}.cmd"));
        #[cfg(not(windows))]
        let path = dir.join(base_name);

        #[cfg(windows)]
        let content = _windows_body;
        #[cfg(not(windows))]
        let content = unix_body;

        fs::write(&path, content).expect("write mock executable");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path).expect("meta").permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).expect("chmod");
        }

        path
    }

    #[test]
    fn test_session_key_for_backend() {
        assert_eq!(session_key_for_backend("codex"), "codex_session");
        assert_eq!(session_key_for_backend("claude"), "claude_session");
    }

    #[test]
    fn test_session_key_for_backend_role() {
        assert_eq!(
            session_key_for_backend_role("codex", Some("reviewer")),
            "reviewer_codex_session"
        );
        assert_eq!(
            session_key_for_backend_role("claude", Some("coder")),
            "coder_claude_session"
        );
        assert_eq!(
            session_key_for_backend_role("claude", Some("unknown")),
            "claude_session"
        );
    }

    #[test]
    fn test_extract_session_id() {
        let text = "pid=9999\nSESSION_ID: 123456\n";
        assert_eq!(extract_session_id(text).as_deref(), Some("123456"));
    }

    #[test]
    fn test_resolve_wrapper_from_env_path() {
        let dir = tempdir().expect("tempdir");
        let bin = write_mock_executable(
            dir.path(),
            "wrapper",
            "#!/bin/bash\necho ok\n",
            "@echo off\r\necho ok\r\n",
        );

        let result =
            resolve_wrapper_bin(Some(&bin.to_string_lossy()), dir.path()).expect("resolve");
        assert_eq!(result.bin, bin);
    }

    #[test]
    fn test_run_backend_mock() {
        let dir = tempdir().expect("tempdir");
        let wrapper = write_mock_executable(
            dir.path(),
            "codeagent-wrapper",
            "#!/bin/bash\necho \"mock backend:$2\"\necho \"SESSION_ID: 123456\"\n",
            "@echo off\r\necho mock backend:%2\r\necho SESSION_ID: 123456\r\n",
        );

        let result = run_backend(&wrapper, "codex", "hello", None, dir.path(), None).expect("run");
        assert_eq!(result.exit_code, 0);
        assert!(result.output.contains("SESSION_ID: 123456"));
    }
}

use anyhow::Result;
use fusion_runtime_io::{read_text, write_text};
use std::path::Path;

const ENGINE_COMMENT: &str = "# rust primary control plane";
const DEFAULT_CONFIG_VERSION: &str = "2.6.3";

pub(crate) fn default_config_text(engine: &str) -> String {
    format!(
        "runtime:\n  enabled: true\n  compat_mode: true\n  engine: \"{engine}\"  {ENGINE_COMMENT}\n  version: \"{DEFAULT_CONFIG_VERSION}\"\n\nbackends:\n  primary: codex\n  fallback: claude\n\nagents:\n  enabled: false\n  mode: single_orchestrator  # default; role_handoff is also supported\n  review_policy: high_risk\n  explain_level: compact\n"
    )
}

pub(crate) fn ensure_config_runtime_engine(config_path: &Path, engine: &str) -> Result<()> {
    let content = if config_path.is_file() {
        read_text(config_path)?
    } else {
        default_config_text(engine)
    };
    let updated = normalize_config_runtime_engine(&content, engine);
    write_text(config_path, &updated)
}

pub(crate) fn normalize_config_runtime_engine(content: &str, engine: &str) -> String {
    let mut output = Vec::new();
    let mut saw_runtime = false;
    let mut in_runtime = false;
    let mut runtime_indent = 0usize;
    let mut engine_written = false;

    for line in content.lines() {
        let trimmed_start = line.trim_start();
        let trimmed = line.trim();
        let indent = line.len().saturating_sub(trimmed_start.len());

        if in_runtime
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && indent <= runtime_indent
            && trimmed != "runtime:"
        {
            if !engine_written {
                output.push(render_engine_line(runtime_indent + 2, engine));
                engine_written = true;
            }
            in_runtime = false;
        }

        if !in_runtime && trimmed == "runtime:" {
            saw_runtime = true;
            in_runtime = true;
            runtime_indent = indent;
            engine_written = false;
            output.push(line.to_string());
            continue;
        }

        if in_runtime && !trimmed.starts_with('#') && trimmed_start.starts_with("engine:") {
            output.push(render_engine_line(indent, engine));
            engine_written = true;
            continue;
        }

        output.push(line.to_string());
    }

    if in_runtime && !engine_written {
        output.push(render_engine_line(runtime_indent + 2, engine));
    }

    if !saw_runtime {
        if !output.is_empty() && !output.last().is_some_and(|line| line.is_empty()) {
            output.push(String::new());
        }
        output.extend([
            "runtime:".to_string(),
            "  enabled: true".to_string(),
            "  compat_mode: true".to_string(),
            render_engine_line(2, engine),
            format!("  version: \"{DEFAULT_CONFIG_VERSION}\""),
        ]);
    }

    let mut updated = output.join("\n");
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated
}

fn render_engine_line(indent: usize, engine: &str) -> String {
    format!(
        "{space:indent$}engine: \"{engine}\"  {ENGINE_COMMENT}",
        space = "",
        indent = indent
    )
}

#[cfg(test)]
mod tests {
    use super::normalize_config_runtime_engine;

    #[test]
    fn replaces_existing_runtime_engine_line() {
        let content = r#"runtime:
  enabled: true
  engine: "legacy"
"#;
        let updated = normalize_config_runtime_engine(content, "rust");

        assert!(updated.contains(r#"engine: "rust"  # rust primary control plane"#));
        assert!(!updated.contains(
            r#"engine: "legacy"
"#
        ));
    }

    #[test]
    fn injects_runtime_engine_when_runtime_block_exists() {
        let content = r#"runtime:
  enabled: true
  compat_mode: true
backends:
  primary: codex
"#;
        let updated = normalize_config_runtime_engine(content, "rust");

        assert!(updated.contains(
            r#"runtime:
  enabled: true
  compat_mode: true
  engine: "rust"  # rust primary control plane
backends:"#
        ));
    }

    #[test]
    fn appends_runtime_block_when_missing() {
        let content = r#"backends:
  primary: codex
"#;
        let updated = normalize_config_runtime_engine(content, "rust");

        assert!(updated.contains(
            r#"backends:
  primary: codex

runtime:
  enabled: true
  compat_mode: true
  engine: "rust"  # rust primary control plane
  version: "2.6.3"
"#
        ));
    }
}

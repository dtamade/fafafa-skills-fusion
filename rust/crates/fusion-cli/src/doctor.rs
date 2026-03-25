use anyhow::{anyhow, Result};
use fusion_runtime_io::{load_flat_config, read_json, read_text};
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::render::read_task_counts;

const PRETOOL_HOOK: &str = "${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-pretool.sh";
const POSTTOOL_HOOK: &str = "${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-posttool.sh";
const STOP_HOOK: &str = "${CLAUDE_PROJECT_DIR:-.}/scripts/fusion-stop-guard.sh";

struct DoctorContext {
    fusion_root: PathBuf,
    project_root: PathBuf,
    fusion_dir: PathBuf,
    json: bool,
    fix: bool,
    fixed: bool,
    ok_count: i64,
    warn_count: i64,
}

impl DoctorContext {
    fn ok(&mut self, message: &str) {
        self.ok_count += 1;
        if !self.json {
            println!("[OK] {message}");
        }
    }

    fn warn(&mut self, message: &str) {
        self.warn_count += 1;
        if !self.json {
            println!("[WARN] {message}");
        }
    }

    fn section(&self, title: &str) {
        if !self.json {
            println!();
            println!("=== {title} ===");
        }
    }
}

pub(crate) fn cmd_doctor(
    project_root: Option<&Path>,
    json_mode: bool,
    fix_mode: bool,
) -> Result<()> {
    let project_root = project_root
        .map(Path::to_path_buf)
        .unwrap_or_else(|| env::current_dir().expect("current dir"));

    if !project_root.is_dir() {
        if json_mode {
            println!(
                "{}",
                serde_json::to_string(&json!({
                    "result": "error",
                    "reason": format!("project_root not found: {}", project_root.display()),
                }))?
            );
        }
        return Err(anyhow!(
            "project_root not found: {}",
            project_root.display()
        ));
    }

    let project_root = fs::canonicalize(project_root)?;
    let fusion_root = detect_fusion_root(&project_root)
        .ok_or_else(|| anyhow!("unable to locate fusion root for doctor command"))?;
    let fusion_dir = project_root.join(".fusion");

    let mut ctx = DoctorContext {
        fusion_root,
        project_root,
        fusion_dir,
        json: json_mode,
        fix: fix_mode,
        fixed: false,
        ok_count: 0,
        warn_count: 0,
    };

    run_doctor(&mut ctx)?;

    if ctx.json {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "result": if ctx.warn_count > 0 { "warn" } else { "ok" },
                "project_root": ctx.project_root.display().to_string(),
                "fusion_root": ctx.fusion_root.display().to_string(),
                "ok_count": ctx.ok_count,
                "warn_count": ctx.warn_count,
                "fixed": ctx.fixed,
            }))?
        );
    }

    if ctx.warn_count > 0 {
        return Err(anyhow!("doctor reported warnings"));
    }

    Ok(())
}

fn run_doctor(ctx: &mut DoctorContext) -> Result<()> {
    ctx.section("Context");
    if !ctx.json {
        println!("fusion_root: {}", ctx.fusion_root.display());
        println!("project_root: {}", ctx.project_root.display());
    }

    ctx.section("Script Presence");
    for script in [
        "fusion-pretool.sh",
        "fusion-posttool.sh",
        "fusion-stop-guard.sh",
    ] {
        let path = ctx.fusion_root.join("scripts").join(script);
        if path.is_file() {
            ctx.ok(&format!("script present: {}", path.display()));
        } else {
            ctx.warn(&format!("missing script: {}", path.display()));
        }
    }

    if ctx.fix {
        match write_project_hooks_settings(&ctx.project_root) {
            Ok(()) => {
                ctx.fixed = true;
                ctx.ok(&format!(
                    "auto-fixed project hooks: {}",
                    ctx.project_root
                        .join(".claude/settings.local.json")
                        .display()
                ));
            }
            Err(error) => {
                ctx.warn(&format!(
                    "auto-fix failed: unable to write {} ({error})",
                    ctx.project_root
                        .join(".claude/settings.local.json")
                        .display()
                ));
            }
        }
    }

    ctx.section("Hook Wiring");
    let project_settings_local = ctx.project_root.join(".claude/settings.local.json");
    let project_settings = ctx.project_root.join(".claude/settings.json");
    let global_settings = home_dir()
        .map(|home| home.join(".claude/settings.json"))
        .unwrap_or_else(|| PathBuf::from(".claude/settings.json"));

    let mut found_project_hooks = false;
    if has_all_hook_names(&project_settings_local) {
        found_project_hooks = true;
        if has_canonical_hook_paths(&project_settings_local) {
            ctx.ok(&format!(
                "project local hooks wired: {}",
                project_settings_local.display()
            ));
        } else {
            ctx.warn("project local hooks use relative or legacy command paths");
        }
    } else if has_all_hook_names(&project_settings) {
        found_project_hooks = true;
        if has_canonical_hook_paths(&project_settings) {
            ctx.ok(&format!(
                "project hooks wired: {}",
                project_settings.display()
            ));
        } else {
            ctx.warn("project hooks use relative or legacy command paths");
        }
    } else {
        ctx.warn("project settings missing full Fusion hook trio (.claude/settings*.json)");
    }

    if contains_any_hook_name(&global_settings) {
        ctx.ok(&format!(
            "global settings contains Fusion hooks: {}",
            global_settings.display()
        ));
    } else if found_project_hooks {
        ctx.ok(&format!(
            "global settings has no Fusion hooks (project hooks are active): {}",
            global_settings.display()
        ));
    } else {
        ctx.warn(&format!(
            "global settings has no Fusion hooks: {}",
            global_settings.display()
        ));
    }

    ctx.section("Workflow State");
    if ctx.fusion_dir.is_dir() {
        ctx.ok(&format!(
            "fusion workspace exists: {}",
            ctx.fusion_dir.display()
        ));
    } else {
        ctx.warn(&format!(
            "fusion workspace missing: {}",
            ctx.fusion_dir.display()
        ));
    }

    let status = read_status(&ctx.fusion_dir.join("sessions.json"));
    if !ctx.json {
        println!("sessions.status: {}", status.as_deref().unwrap_or(""));
    }
    if status.is_none() {
        ctx.warn("sessions.status unavailable (.fusion/sessions.json missing or invalid)");
    }

    let cfg = load_flat_config(&ctx.fusion_dir);
    if !ctx.json {
        println!("runtime.engine: {}", cfg.runtime_engine);
        println!("runtime.compat_mode: {}", cfg.runtime_compat_mode);
    }

    let counts = read_task_counts(&ctx.fusion_dir)?;
    if !ctx.json {
        println!(
            "task_counts: pending={} in_progress={} completed={}",
            counts.pending, counts.in_progress, counts.completed
        );
    }

    Ok(())
}

fn detect_fusion_root(project_root: &Path) -> Option<PathBuf> {
    let exe = env::current_exe().ok();
    if let Some(exe_path) = exe {
        for ancestor in exe_path.ancestors() {
            if ancestor.join("scripts/fusion-pretool.sh").is_file() {
                return Some(ancestor.to_path_buf());
            }
        }
    }

    for ancestor in project_root.ancestors() {
        if ancestor.join("scripts/fusion-pretool.sh").is_file() {
            return Some(ancestor.to_path_buf());
        }
    }

    None
}

fn write_project_hooks_settings(project_root: &Path) -> Result<()> {
    let settings_dir = project_root.join(".claude");
    fs::create_dir_all(&settings_dir)?;
    fs::write(
        settings_dir.join("settings.local.json"),
        format!(
            concat!(
                "{{\n",
                "  \"hooks\": {{\n",
                "    \"PreToolUse\": [\n",
                "      {{\n",
                "        \"matcher\": \"Write|Edit|Bash|Read|Glob|Grep\",\n",
                "        \"hooks\": [\n",
                "          {{\n",
                "            \"type\": \"command\",\n",
                "            \"command\": \"bash \\\"{}\\\"\"\n",
                "          }}\n",
                "        ]\n",
                "      }}\n",
                "    ],\n",
                "    \"PostToolUse\": [\n",
                "      {{\n",
                "        \"matcher\": \"Write|Edit\",\n",
                "        \"hooks\": [\n",
                "          {{\n",
                "            \"type\": \"command\",\n",
                "            \"command\": \"bash \\\"{}\\\"\"\n",
                "          }}\n",
                "        ]\n",
                "      }}\n",
                "    ],\n",
                "    \"Stop\": [\n",
                "      {{\n",
                "        \"hooks\": [\n",
                "          {{\n",
                "            \"type\": \"command\",\n",
                "            \"command\": \"bash \\\"{}\\\"\"\n",
                "          }}\n",
                "        ]\n",
                "      }}\n",
                "    ]\n",
                "  }}\n",
                "}}\n"
            ),
            PRETOOL_HOOK, POSTTOOL_HOOK, STOP_HOOK
        ),
    )?;
    Ok(())
}

fn has_all_hook_names(path: &Path) -> bool {
    let Ok(content) = read_text(path) else {
        return false;
    };
    content.contains("fusion-pretool.sh")
        && content.contains("fusion-posttool.sh")
        && content.contains("fusion-stop-guard.sh")
}

fn contains_any_hook_name(path: &Path) -> bool {
    let Ok(content) = read_text(path) else {
        return false;
    };
    content.contains("fusion-pretool.sh") || content.contains("fusion-stop-guard.sh")
}

fn has_canonical_hook_paths(path: &Path) -> bool {
    let Ok(content) = read_text(path) else {
        return false;
    };
    content.contains(PRETOOL_HOOK) && content.contains(POSTTOOL_HOOK) && content.contains(STOP_HOOK)
}

fn read_status(path: &Path) -> Option<String> {
    let value = read_json(path).ok()?;
    value.get("status")?.as_str().map(ToOwned::to_owned)
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

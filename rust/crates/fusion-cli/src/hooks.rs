use anyhow::Result;
use fusion_runtime_io::{json_set_string, read_json, write_json_pretty};
use serde_json::{Map, Value};
use std::path::Path;

use crate::render::truncate_chars;

pub(crate) use crate::posttool::cmd_hook_posttool;
pub(crate) use crate::pretool::cmd_hook_pretool;
pub(crate) use crate::stop_guard::evaluate_stop_guard;

pub(crate) fn cmd_hook_stop_guard(fusion_dir: &Path) -> Result<()> {
    let output = evaluate_stop_guard(fusion_dir)?;
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

pub(crate) fn cmd_hook_set_goal(fusion_dir: &Path, goal: &str) -> Result<()> {
    let sessions_path = fusion_dir.join("sessions.json");
    let mut data = if sessions_path.is_file() {
        read_json(&sessions_path)?
    } else {
        Value::Object(Map::new())
    };
    json_set_string(&mut data, "goal", goal);
    write_json_pretty(&sessions_path, &data)?;
    println!("Goal set: {}...", truncate_chars(goal.to_string(), 60));
    Ok(())
}

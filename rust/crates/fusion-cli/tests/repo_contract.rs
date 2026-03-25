use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    fs::read_to_string(repo_root().join(relative)).expect("read repository file")
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn contains_normalized(haystack: &str, needle: &str) -> bool {
    normalize_whitespace(haystack).contains(&normalize_whitespace(needle))
}

fn line_contains_normalized(haystack: &str, needle: &str) -> bool {
    haystack
        .lines()
        .any(|line| normalize_whitespace(line).contains(&normalize_whitespace(needle)))
}

fn line_equals_normalized(haystack: &str, needle: &str) -> bool {
    haystack
        .lines()
        .any(|line| normalize_whitespace(line) == normalize_whitespace(needle))
}

fn appears_before_normalized(haystack: &str, first: &str, second: &str) -> bool {
    let normalized_haystack = normalize_whitespace(haystack);
    let normalized_first = normalize_whitespace(first);
    let normalized_second = normalize_whitespace(second);

    match (
        normalized_haystack.find(&normalized_first),
        normalized_haystack.find(&normalized_second),
    ) {
        (Some(first_idx), Some(second_idx)) => first_idx < second_idx,
        _ => false,
    }
}

fn find_line_offset_normalized(content: &str, needle: &str) -> usize {
    let normalized_needle = normalize_whitespace(needle.trim_end_matches('\n'));
    let mut offset = 0;

    for line in content.split_inclusive('\n') {
        let line_without_newline = line.trim_end_matches('\n');
        if normalize_whitespace(line_without_newline) == normalized_needle {
            return offset;
        }
        offset += line.len();
    }

    panic!("missing normalized line marker: {needle}");
}

fn is_generated_cache_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | ".fusion" | "target" | ".ace-tool" | ".cargo-codex"
    )
}

fn retired_lang_word() -> String {
    ["py", "thon"].concat()
}

fn retired_lang_title() -> String {
    ["Py", "thon"].concat()
}

fn retired_test_runner_word() -> String {
    ["py", "test"].concat()
}

fn retired_test_command() -> String {
    [retired_test_runner_word(), " -q".to_string()].concat()
}

fn retired_cache_dir() -> String {
    [
        ".".to_string(),
        retired_test_runner_word(),
        "_cache".to_string(),
    ]
    .concat()
}

fn retired_setup_action() -> String {
    ["actions/setup-".to_string(), retired_lang_word()].concat()
}

fn retired_install_step() -> String {
    [
        "pi".to_string(),
        "p install ".to_string(),
        retired_test_runner_word(),
    ]
    .concat()
}

fn retired_package_installer() -> String {
    ["pi".to_string(), "p install".to_string()].concat()
}

fn retired_module_invocation() -> String {
    [retired_lang_word(), " -m ".to_string()].concat()
}

fn retired_isolated_env_word() -> String {
    ["virtual".to_string(), "env".to_string()].concat()
}

fn retired_short_env_word() -> String {
    ["ve".to_string(), "nv".to_string()].concat()
}

fn retired_bytecode_cache_dir() -> String {
    ["__".to_string(), ["p", "y"].concat(), "cache__".to_string()].concat()
}

fn retired_type_cache_dir() -> String {
    [
        ".".to_string(),
        "my".to_string(),
        ["p", "y"].concat(),
        "_cache".to_string(),
    ]
    .concat()
}

fn retired_project_file_names() -> Vec<String> {
    vec![
        [retired_lang_word(), "project.toml".to_string()].concat(),
        ["require".to_string(), "ments.txt".to_string()].concat(),
        [
            "require".to_string(),
            "ments-".to_string(),
            "dev.txt".to_string(),
        ]
        .concat(),
        [
            "require".to_string(),
            "ments-".to_string(),
            "test.txt".to_string(),
        ]
        .concat(),
        ["set".to_string(), "up.".to_string(), ["p", "y"].concat()].concat(),
        ["set".to_string(), "up.cfg".to_string()].concat(),
        ["to".to_string(), "x.ini".to_string()].concat(),
        ".".to_string() + &retired_lang_word() + "-version",
        ["Pip".to_string(), "file".to_string()].concat(),
        ["Pip".to_string(), "file.lock".to_string()].concat(),
        ["poe".to_string(), "try.lock".to_string()].concat(),
        ["my".to_string(), ["p", "y"].concat(), ".ini".to_string()].concat(),
        [".".to_string(), "flake".to_string(), "8".to_string()].concat(),
        retired_test_runner_word() + ".ini",
        ["environ".to_string(), "ment.yml".to_string()].concat(),
        ["environ".to_string(), "ment.yaml".to_string()].concat(),
        ["con".to_string(), "da.yml".to_string()].concat(),
        ["con".to_string(), "da.yaml".to_string()].concat(),
    ]
}

fn retired_project_dir_names() -> Vec<String> {
    vec![
        ".".to_string() + &retired_short_env_word(),
        retired_short_env_word(),
        retired_bytecode_cache_dir(),
        retired_type_cache_dir(),
        [".".to_string(), "to".to_string(), "x".to_string()].concat(),
        [".".to_string(), "no".to_string(), "x".to_string()].concat(),
        [".".to_string(), "ru".to_string(), "ff_cache".to_string()].concat(),
        [
            ".".to_string(),
            "ipy".to_string(),
            "nb_checkpoints".to_string(),
        ]
        .concat(),
    ]
}

fn retired_distribution_dir_suffixes() -> Vec<String> {
    vec![
        [".".to_string(), "egg".to_string(), "-info".to_string()].concat(),
        [".".to_string(), "dist".to_string(), "-info".to_string()].concat(),
    ]
}

fn retired_distribution_file_suffixes() -> Vec<String> {
    vec![
        [".".to_string(), "whl".to_string()].concat(),
        [".".to_string(), "egg".to_string()].concat(),
        [".".to_string(), "ipy".to_string(), "nb".to_string()].concat(),
    ]
}

fn retired_vendor_dir_names() -> Vec<String> {
    vec![[
        ["site".to_string(), "-".to_string()].concat(),
        "packages".to_string(),
    ]
    .concat()]
}

fn retired_skip_flag() -> String {
    ["--skip-".to_string(), retired_lang_word()].concat()
}

fn retired_version_label() -> String {
    [retired_lang_title(), " 3.10+".to_string()].concat()
}

fn retired_version_field() -> String {
    [retired_lang_title(), " version".to_string()].concat()
}

fn walk_retired_source_files(root: &Path, current: &Path, found: &mut Vec<String>) {
    for entry in fs::read_dir(current).expect("read dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();

        if path.is_dir() {
            let skip = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(is_generated_cache_dir);
            if !skip {
                walk_retired_source_files(root, &path, found);
            }
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) == Some("py") {
            found.push(
                path.strip_prefix(root)
                    .expect("strip prefix")
                    .display()
                    .to_string(),
            );
        }
    }
}

fn job_block<'a>(content: &'a str, start: &str, end: Option<&str>) -> &'a str {
    let start_idx = find_line_offset_normalized(content, start);
    let rest = &content[start_idx..];
    match end {
        Some(end_marker) => {
            let end_idx = find_line_offset_normalized(rest, end_marker);
            &rest[..end_idx]
        }
        None => rest,
    }
}

fn walk_files_with_extensions(
    root: &Path,
    current: &Path,
    allowed_extensions: &[&str],
    found: &mut Vec<String>,
) {
    for entry in fs::read_dir(current).expect("read dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();

        if path.is_dir() {
            let skip = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(is_generated_cache_dir);
            if !skip {
                walk_files_with_extensions(root, &path, allowed_extensions, found);
            }
            continue;
        }

        let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        if allowed_extensions.contains(&ext) {
            found.push(
                path.strip_prefix(root)
                    .expect("strip prefix")
                    .display()
                    .to_string(),
            );
        }
    }
}

#[test]
fn repository_contains_no_retired_source_files() {
    let root = repo_root();
    let mut retired_source_files = Vec::new();
    walk_retired_source_files(&root, &root, &mut retired_source_files);
    assert_eq!(
        retired_source_files,
        Vec::<String>::new(),
        "expected repository to contain no tracked legacy source files"
    );
}

#[test]
fn repository_contains_no_retired_cache_dirs() {
    fn walk_retired_cache_dirs(root: &Path, current: &Path, found: &mut Vec<String>) {
        for entry in fs::read_dir(current).expect("read dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            if is_generated_cache_dir(name) {
                continue;
            }

            let retired_cache = retired_cache_dir();
            let retired_bytecode_cache = retired_bytecode_cache_dir();
            let retired_type_cache = retired_type_cache_dir();
            if name == retired_cache || name == retired_bytecode_cache || name == retired_type_cache
            {
                found.push(
                    path.strip_prefix(root)
                        .expect("strip prefix")
                        .display()
                        .to_string(),
                );
                continue;
            }

            walk_retired_cache_dirs(root, &path, found);
        }
    }

    let root = repo_root();
    let mut retired_cache_dirs = Vec::new();
    walk_retired_cache_dirs(&root, &root, &mut retired_cache_dirs);
    assert_eq!(
        retired_cache_dirs,
        Vec::<String>::new(),
        "expected repository to contain no retired cache directories"
    );
}

#[test]
fn repository_contains_no_retired_project_artifacts() {
    fn walk_retired_project_artifacts(root: &Path, current: &Path, found: &mut Vec<String>) {
        let retired_files = retired_project_file_names();
        let retired_dirs = retired_project_dir_names();
        let retired_distribution_dirs = retired_distribution_dir_suffixes();
        let retired_distribution_files = retired_distribution_file_suffixes();
        let retired_vendor_dirs = retired_vendor_dir_names();

        for entry in fs::read_dir(current).expect("read dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");

            if path.is_dir() {
                if is_generated_cache_dir(name) {
                    continue;
                }
                if retired_dirs.iter().any(|retired| retired == name)
                    || retired_vendor_dirs.iter().any(|retired| retired == name)
                    || retired_distribution_dirs
                        .iter()
                        .any(|retired| name.ends_with(retired))
                {
                    found.push(
                        path.strip_prefix(root)
                            .expect("strip prefix")
                            .display()
                            .to_string(),
                    );
                    continue;
                }
                walk_retired_project_artifacts(root, &path, found);
                continue;
            }

            if retired_files.iter().any(|retired| retired == name)
                || retired_distribution_files
                    .iter()
                    .any(|retired| name.ends_with(retired))
            {
                found.push(
                    path.strip_prefix(root)
                        .expect("strip prefix")
                        .display()
                        .to_string(),
                );
            }
        }
    }

    let root = repo_root();
    let mut retired_project_artifacts = Vec::new();
    walk_retired_project_artifacts(&root, &root, &mut retired_project_artifacts);
    assert_eq!(
        retired_project_artifacts,
        Vec::<String>::new(),
        "expected repository to contain no retired project artifact files or directories"
    );
}

#[test]
fn repository_contains_no_retired_runtime_dirs() {
    let root = repo_root();

    for relative in [
        "scripts/runtime",
        "scripts/runtime/_reference",
        "scripts/runtime/tests",
    ] {
        assert!(
            !root.join(relative).exists(),
            "{relative} should be removed once Rust-only convergence is complete"
        );
    }
}

#[test]
fn active_surface_stays_free_of_retired_tooling_words() {
    fn assert_no_retired_tooling_terms(relative: &str) {
        let lower = read(relative).to_lowercase();
        for retired_term in [
            retired_lang_word(),
            retired_test_runner_word(),
            retired_setup_action(),
            retired_package_installer(),
            retired_module_invocation(),
            retired_short_env_word(),
            retired_isolated_env_word(),
        ] {
            assert!(
                !lower.contains(&retired_term),
                "{relative} should not reference retired tooling term: {retired_term}"
            );
        }
    }

    for relative in [
        "README.md",
        "README.zh-CN.md",
        "CHANGELOG.md",
        "CONTRIBUTING.md",
        "CONTRIBUTING.zh-CN.md",
        "EXECUTION_PROTOCOL.md",
        "PARALLEL_EXECUTION.md",
        "ROADMAP.md",
        "SESSION_RECOVERY.md",
        "SKILL.md",
        "rust/README.md",
        "templates/config.yaml",
        "docs/CLI_CONTRACT_MATRIX.md",
        "docs/COMPATIBILITY.md",
        "docs/E2E_EXAMPLE.md",
        "docs/HOOKS_SETUP.md",
        "docs/REPO_CONVERGENCE_SUMMARY_2026-03.md",
        "docs/REPO_HYGIENE.md",
        "docs/RUNTIME_KERNEL_DESIGN.md",
        "docs/RUST_FUSION_BRIDGE_ROADMAP.md",
        "docs/UPGRADE_v2_COMPAT.md",
    ] {
        assert_no_retired_tooling_terms(relative);
    }

    let root = repo_root();

    let mut github_files = Vec::new();
    walk_files_with_extensions(
        &root,
        &root.join(".github"),
        &["md", "yml", "yaml"],
        &mut github_files,
    );
    github_files.sort();
    github_files.dedup();
    for relative in github_files {
        assert_no_retired_tooling_terms(&relative);
    }

    let mut script_files = Vec::new();
    walk_files_with_extensions(&root, &root.join("scripts"), &["sh"], &mut script_files);
    script_files.sort();
    script_files.dedup();
    for relative in script_files {
        assert_no_retired_tooling_terms(&relative);
    }
}

#[test]
fn active_docs_and_templates_use_rust_only_validation() {
    let active_files = [
        "README.md",
        "README.zh-CN.md",
        "ROADMAP.md",
        "templates/config.yaml",
        "rust/README.md",
        "PARALLEL_EXECUTION.md",
        "docs/E2E_EXAMPLE.md",
        "docs/HOOKS_SETUP.md",
        "docs/UPGRADE_v2_COMPAT.md",
        "docs/CLI_CONTRACT_MATRIX.md",
        "docs/COMPATIBILITY.md",
        "SESSION_RECOVERY.md",
        "EXECUTION_PROTOCOL.md",
        "docs/RUNTIME_KERNEL_DESIGN.md",
        "docs/RUST_FUSION_BRIDGE_ROADMAP.md",
        "docs/REPO_CONVERGENCE_SUMMARY_2026-03.md",
    ];

    for relative in active_files {
        let content = read(relative);
        let lower = content.to_lowercase();
        let retired_lang = retired_lang_word();
        let retired_test_runner = retired_test_runner_word();
        assert!(
            !lower.contains(&retired_lang),
            "{relative} should not reference removed implementation wording"
        );
        assert!(
            !lower.contains(&retired_test_runner),
            "{relative} should not reference retired test-runner wording"
        );
        assert!(
            !content.contains("tests/runtime/"),
            "{relative} should not reference removed legacy test paths"
        );
        assert!(
            !content.contains("scripts/runtime"),
            "{relative} should not reference removed legacy runtime paths"
        );
        assert!(
            !content.contains("compat_v2"),
            "{relative} should not reference removed legacy adapter names"
        );
        assert!(
            !content.contains("fusion-explain.sh"),
            "{relative} should not reference removed shell entrypoints"
        );
        assert!(
            !content.contains("fusion-doctor.sh"),
            "{relative} should not reference removed shell entrypoints"
        );
        assert!(
            !content.contains("RUNTIME_KERNEL.md"),
            "{relative} should not reference removed planned doc filenames"
        );
        assert!(
            !content.contains("MODEL_BUS.md"),
            "{relative} should not reference removed planned doc filenames"
        );
        assert!(
            !content.contains("OPERATIONS_RUNBOOK.md"),
            "{relative} should not reference removed planned doc filenames"
        );
        assert!(
            !content.contains("UPGRADE_v2_to_v3.md"),
            "{relative} should not reference removed planned doc filenames"
        );
        assert!(
            !content.contains("create_kernel()"),
            "{relative} should not reference removed implementation symbols"
        );
        assert!(
            !content.contains("legacy_compat"),
            "{relative} should not reference removed implementation symbols"
        );
        assert!(
            !content.contains("parallel_bench"),
            "{relative} should not reference removed implementation symbols"
        );
        assert!(
            !content.contains("hook_latency_bench"),
            "{relative} should not reference removed implementation symbols"
        );
        assert!(
            !content.contains("model_bus.mode=shadow"),
            "{relative} should not reference removed implementation symbols"
        );
        assert!(
            !content.contains("jq/grep"),
            "{relative} should not reference retired historical shorthand"
        );
        assert!(
            !content.contains("shadow/canary"),
            "{relative} should not reference retired historical shorthand"
        );
        assert!(
            !content.contains("283/283"),
            "{relative} should not reference stale historical test counts"
        );
        assert!(
            !content.contains("60/60"),
            "{relative} should not reference stale historical test counts"
        );
        assert!(
            !content.contains("20/20"),
            "{relative} should not reference stale historical test counts"
        );
        for retired_line in [
            "| pretool | ~5ms | ~0.3ms (bridge / compat path) | 更快 |",
            "| posttool | ~5ms | ~0.2ms (bridge / compat path) | 更快 |",
            "| stop-guard | ~10ms | ~0.4ms (bridge / compat path) | 更快 |",
            "| DAG 依赖违规数 | = 0 | 0 | ✅ |",
            "| 中位加速比 | ≥ 1.4x | 2.00x | ✅ |",
            "| 冲突回滚率 | ≤ 5% | 4.8% | ✅ |",
            "| Token 超支率 | ≤ 10% | 6.0% | ✅ |",
            "| 硬上限突破 | = 0 | 0 | ✅ |",
            "| 调度决策 p95 | < 200ms | 0.09ms | ✅ |",
            "| v2.1.0 回归 | 139/139 | 139/139 | ✅ |",
            "| FSM + Kernel (v2.1.0) | 139 | 100% |",
            "| DAG 任务图 | 39 | 100% |",
            "| 冲突检测 | 15 | 100% |",
            "| 预算管理 | 24 | 100% |",
            "| 模型路由 | 12 | 100% |",
            "| 调度器 | 16 | 100% |",
            "| 调度器集成 | 23 | 100% |",
            "| **Total** | **268** | **100%** |",
            "| 冲突检测误报 | 支持\"一键回退串行\" | ✅ 冲突回滚率 4.8% |",
        ] {
            assert!(
                !line_contains_normalized(&content, retired_line),
                "{relative} should not reference stale historical test count tables"
            );
        }
    }

    let readme = read("README.md");
    assert!(contains_normalized(&readme, "cargo test --release"));
    assert!(contains_normalized(
        &readme,
        "old runtime/reference layer has been removed from the repository."
    ));
    assert!(contains_normalized(
        &readme,
        "Use `scripts/fusion-init.sh` or `fusion-bridge init` to generate `.fusion/config.yaml` from `templates/config.yaml`"
    ));
    assert!(contains_normalized(&readme, "scheduler: enabled: true"));
    assert!(contains_normalized(
        &readme,
        "including `scheduler.enabled: true` as the current default"
    ));
    assert!(contains_normalized(
        &readme,
        "Local Rust caches such as `rust/target/` and `rust/.cargo-codex/` are generated machine state"
    ));
    assert!(contains_normalized(
        &readme,
        "Host-local tool settings such as `.ace-tool/`, `.claude/settings.json`, and `.claude/settings.local.json`"
    ));
    assert!(contains_normalized(
        &readme,
        "`.claude/settings.example.json` remains the checked-in hook template"
    ));
    assert!(contains_normalized(
        &readme,
        "checked-in hook template; copy it to your host-local `.claude/settings.json`"
    ));
    assert!(contains_normalized(
        &readme,
        "[`docs/V3_GA_EXECUTION_ROADMAP.md`](docs/V3_GA_EXECUTION_ROADMAP.md)"
    ));
    assert!(contains_normalized(
        &readme,
        "current v3 GA execution roadmap"
    ));
    assert!(appears_before_normalized(
        &readme,
        "[`docs/V3_GA_EXECUTION_ROADMAP.md`](docs/V3_GA_EXECUTION_ROADMAP.md)",
        "[`docs/RUST_FUSION_BRIDGE_ROADMAP.md`](docs/RUST_FUSION_BRIDGE_ROADMAP.md)"
    ));
    assert!(contains_normalized(
        &readme,
        "If active docs or repository/runtime contracts change, update `rust/crates/fusion-cli/tests/repo_contract.rs` together with the affected contract docs."
    ));
    assert!(contains_normalized(
        &readme,
        "Review-Status: none|pending|approved|changes_requested"
    ));
    assert!(contains_normalized(&readme, "role_handoff"));
    assert!(contains_normalized(&readme, "agent_collaboration_mode"));
    assert!(contains_normalized(
        &readme,
        "No alternate runtime engine selection remains on the current control path"
    ));
    assert!(contains_normalized(
        &readme,
        "As of 2026-03-25, macOS and Windows (Git Bash) have fresh remote CI promotion evidence via run `23539348456`"
    ));
    assert!(contains_normalized(
        &readme,
        "WSL is tracked as post-GA evidence rather than a current GA blocker."
    ));
    assert!(contains_normalized(&readme, "`docs/COMPATIBILITY.md`"));
    assert!(contains_normalized(
        &readme,
        "cross-platform smoke summaries"
    ));
    assert!(contains_normalized(
        &readme,
        "cross-platform-smoke-summary.json"
    ));
    assert!(contains_normalized(&readme, "ci-remote-evidence.sh"));
    assert!(contains_normalized(
        &readme,
        "remote-ci-evidence-summary.json"
    ));

    let changelog = read("CHANGELOG.md");
    assert!(contains_normalized(
        &changelog,
        "Repo convergence cleanup: Rust-first control plane, thinner shell layer, stricter repository/runtime contracts."
    ));
    assert!(contains_normalized(
        &changelog,
        "Generated `.fusion/config.yaml` is workspace state initialized from the checked-in `templates/config.yaml` baseline"
    ));
    assert!(contains_normalized(
        &changelog,
        "Checked-in `.claude/settings.example.json` remains the hook template, while `.claude/settings.json` and `.claude/settings.local.json` stay host-local hook configuration"
    ));

    let readme_zh = read("README.zh-CN.md");
    assert!(contains_normalized(&readme_zh, "cargo test --release"));
    assert!(contains_normalized(
        &readme_zh,
        "旧 runtime/reference 层已从仓库移除"
    ));
    assert!(contains_normalized(
        &readme_zh,
        "先运行 `scripts/fusion-init.sh` 或 `fusion-bridge init`，从 `templates/config.yaml` 生成 `.fusion/config.yaml`"
    ));
    assert!(contains_normalized(&readme_zh, "scheduler: enabled: true"));
    assert!(contains_normalized(
        &readme_zh,
        "完整推荐基线请参考 `templates/config.yaml`。"
    ));
    assert!(line_equals_normalized(
        &readme_zh,
        "## 发布状态（2026-03-21）"
    ));
    assert!(contains_normalized(
        &readme_zh,
        "默认配置生成：`scripts/fusion-init.sh` → `fusion-bridge init`"
    ));
    assert!(contains_normalized(&readme_zh, "生成后的工作区配置"));
    assert!(contains_normalized(&readme_zh, "由模板生成的运行时配置"));
    assert!(contains_normalized(&readme_zh, "受版本控制的配置模板"));
    assert!(contains_normalized(
        &readme_zh,
        "`rust/target/`、`rust/.cargo-codex/` 这类本地 Rust cache 属于机器生成状态"
    ));
    assert!(contains_normalized(
        &readme_zh,
        "`.ace-tool/`、`.claude/settings.json`、`.claude/settings.local.json` 这类宿主本地设置"
    ));
    assert!(contains_normalized(
        &readme_zh,
        "`.claude/settings.example.json` 继续作为受版本控制的 Hook 模板"
    ));
    assert!(contains_normalized(
        &readme_zh,
        "[docs/V3_GA_EXECUTION_ROADMAP.md](docs/V3_GA_EXECUTION_ROADMAP.md)"
    ));
    assert!(contains_normalized(&readme_zh, "当前 v3 GA 执行路线图"));
    assert!(appears_before_normalized(
        &readme_zh,
        "[docs/V3_GA_EXECUTION_ROADMAP.md](docs/V3_GA_EXECUTION_ROADMAP.md)",
        "[docs/RUST_FUSION_BRIDGE_ROADMAP.md](docs/RUST_FUSION_BRIDGE_ROADMAP.md)"
    ));
    assert!(contains_normalized(
        &readme_zh,
        "受版本控制的 Hook 模板；复制后生成宿主本地 `.claude/settings.json`"
    ));
    assert!(contains_normalized(
        &readme_zh,
        "如果活文档或仓库/runtime 契约发生变化，请同步更新 `rust/crates/fusion-cli/tests/repo_contract.rs` 以及受影响的契约文档。"
    ));
    assert!(contains_normalized(
        &readme_zh,
        "Review-Status: none|pending|approved|changes_requested"
    ));
    assert!(contains_normalized(&readme_zh, "role_handoff"));
    assert!(contains_normalized(&readme_zh, "agent_collaboration_mode"));
    assert!(contains_normalized(
        &readme_zh,
        "截至 2026-03-25，macOS 与 Windows (Git Bash) 已通过远端 CI promotion evidence 升级为已验证状态，对应 run 为 `23539348456`"
    ));
    assert!(contains_normalized(
        &readme_zh,
        "WSL 当前仍按 post-GA 证据跟踪，不是当前 GA 阻断项。"
    ));
    assert!(contains_normalized(
        &readme_zh,
        "[docs/COMPATIBILITY.md](docs/COMPATIBILITY.md)"
    ));
    assert!(contains_normalized(&readme_zh, "跨平台 smoke summary JSON"));
    assert!(contains_normalized(
        &readme_zh,
        "cross-platform-smoke-summary.json"
    ));
    assert!(contains_normalized(&readme_zh, "ci-remote-evidence.sh"));
    assert!(contains_normalized(
        &readme_zh,
        "remote-ci-evidence-summary.json"
    ));
    assert!(!line_equals_normalized(
        &readme_zh,
        "## 发布状态（2026-02-10）"
    ));
    assert!(!contains_normalized(
        &readme_zh,
        "runtime 默认启用：`scripts/fusion-init.sh`"
    ));
    assert!(!contains_normalized(
        &readme_zh,
        "scheduler: enabled: false"
    ));

    let template_config = read("templates/config.yaml");
    assert!(line_equals_normalized(&template_config, "enabled: true"));
    assert!(line_contains_normalized(
        &template_config,
        "compat_mode: true"
    ));
    assert!(line_contains_normalized(
        &template_config,
        "engine: \"rust\""
    ));
    assert!(contains_normalized(
        &template_config,
        "agents: enabled: false"
    ));
    assert!(line_contains_normalized(
        &template_config,
        "mode: single_orchestrator"
    ));
    assert!(contains_normalized(&template_config, "role_handoff"));
    assert!(contains_normalized(
        &template_config,
        "Review-Status: none | pending | approved | changes_requested"
    ));
    assert!(line_contains_normalized(
        &template_config,
        "review_policy: high_risk"
    ));
    assert!(line_contains_normalized(
        &template_config,
        "explain_level: compact"
    ));
    assert!(contains_normalized(
        &template_config,
        "scheduler: enabled: true"
    ));
    assert!(contains_normalized(
        &template_config,
        "safe_backlog: enabled: true"
    ));
    assert!(contains_normalized(
        &template_config,
        "supervisor: enabled: false"
    ));
    assert!(!line_equals_normalized(&template_config, "engine: legacy"));

    let rust_readme = read("rust/README.md");
    assert!(contains_normalized(
        &rust_readme,
        "旧 runtime/reference 层已从仓库移除"
    ));
    assert!(contains_normalized(&rust_readme, "cargo test --release"));
    assert!(contains_normalized(
        &rust_readme,
        "`fusion-bridge init` 会从 `templates/config.yaml` 生成 `.fusion/config.yaml`"
    ));
    assert!(contains_normalized(
        &rust_readme,
        "其中 `.fusion/config.yaml` 是工作区生成配置，`templates/config.yaml` 才是受版本控制基线。"
    ));
    assert!(contains_normalized(
        &rust_readme,
        "`runtime.enabled=true`、`scheduler.enabled=true`、`safe_backlog.enabled=true`"
    ));
    assert!(contains_normalized(
        &rust_readme,
        "`supervisor.enabled=false`"
    ));
    assert!(contains_normalized(
        &rust_readme,
        "当前 live 配置文档不再公开多 runtime engine 选择"
    ));
    assert!(contains_normalized(
        &rust_readme,
        "`rust/target/`、`rust/.cargo-codex/` 属于本地 Rust cache"
    ));
    assert!(contains_normalized(
        &rust_readme,
        "`.claude/settings.example.json` 是受版本控制的 Hook 模板"
    ));
    assert!(contains_normalized(
        &rust_readme,
        "`.claude/settings.local.json`（宿主本地 override 文件，不属于仓库输入）"
    ));
    assert!(contains_normalized(
        &rust_readme,
        "`docs/V3_GA_EXECUTION_ROADMAP.md`"
    ));
    assert!(contains_normalized(&rust_readme, "当前执行真源路线图"));
    assert!(contains_normalized(
        &rust_readme,
        "请同步更新相关活文档，以及 `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`"
    ));

    let parallel_doc = read("PARALLEL_EXECUTION.md");
    assert!(contains_normalized(
        &parallel_doc,
        "`execution.parallel` 与 `scheduler.max_parallel` 共同约束"
    ));
    assert!(contains_normalized(
        &parallel_doc,
        "### `.fusion/config.yaml`"
    ));
    assert!(contains_normalized(
        &parallel_doc,
        "当前推荐基线请参考 `templates/config.yaml`。"
    ));
    assert!(contains_normalized(
        &parallel_doc,
        "请同步更新相关活文档，以及 `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`"
    ));
    assert!(contains_normalized(
        &parallel_doc,
        "scheduler: enabled: true"
    ));
    assert!(!contains_normalized(
        &parallel_doc,
        "由 `config.yaml` 中的 `parallel` 配置控制（默认 2）"
    ));

    let hooks = read("docs/HOOKS_SETUP.md");
    assert!(contains_normalized(
        &hooks,
        "bash scripts/ci-cross-platform-smoke.sh"
    ));
    assert!(contains_normalized(
        &hooks,
        "bash scripts/ci-cross-platform-json-smoke.sh"
    ));
    assert!(contains_normalized(
        &hooks,
        "bash scripts/ci-remote-evidence.sh --json"
    ));
    assert!(contains_normalized(
        &hooks,
        "/tmp/cross-platform-smoke-macos/cross-platform-smoke-summary.json"
    ));
    assert!(contains_normalized(
        &hooks,
        "/tmp/cross-platform-smoke-windows/cross-platform-smoke-summary.json"
    ));
    assert!(contains_normalized(
        &hooks,
        "cargo test --release -p fusion-cli --test shell_contract"
    ));
    assert!(contains_normalized(
        &hooks,
        "Former runtime/reference layer: removed from the repository"
    ));
    assert!(contains_normalized(
        &hooks,
        "当前模板默认值已经启用 `runtime.enabled: true`"
    ));
    assert!(contains_normalized(
        &hooks,
        "完整推荐基线请参考 `templates/config.yaml`"
    ));
    assert!(contains_normalized(
        &hooks,
        "`.claude/settings.example.json` 是受版本控制的模板"
    ));
    assert!(contains_normalized(
        &hooks,
        "实际 `.claude/settings.json` 与 `.claude/settings.local.json` 属于宿主本地 Hook 配置"
    ));
    assert!(contains_normalized(
        &hooks,
        "`.claude/settings.local.json` 也属于宿主本地 override 文件"
    ));
    assert!(contains_normalized(
        &hooks,
        "请同步更新 `rust/crates/fusion-cli/tests/repo_contract.rs`、`rust/crates/fusion-cli/tests/shell_contract.rs` 与本页"
    ));

    let upgrade = read("docs/UPGRADE_v2_COMPAT.md");
    assert!(contains_normalized(
        &upgrade,
        "cargo test --release -p fusion-cli --test repo_contract"
    ));
    assert!(contains_normalized(
        &upgrade,
        "bash scripts/ci-machine-mode-smoke.sh"
    ));
    assert!(contains_normalized(
        &upgrade,
        "旧 runtime/reference 层已从仓库移除"
    ));
    assert!(contains_normalized(&upgrade, "`jq` 不是运行时必需依赖"));
    assert!(line_contains_normalized(&upgrade, "最后校准: 2026-03-21"));
    assert!(contains_normalized(
        &upgrade,
        "当前仓库模板默认值已经切到 `runtime.enabled=true`"
    ));
    assert!(contains_normalized(
        &upgrade,
        "通过由 `templates/config.yaml` 生成的 `.fusion/config.yaml` 控制"
    ));
    assert!(contains_normalized(&upgrade, "由模板生成的工作区配置"));
    assert!(contains_normalized(
        &upgrade,
        "修改 `.fusion/config.yaml` 中 `runtime.enabled: false`"
    ));
    assert!(contains_normalized(
        &upgrade,
        "runtime.enabled=false (历史升级前基线)"
    ));
    assert!(contains_normalized(
        &upgrade,
        "请同步更新受影响的活文档，以及 `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`"
    ));
    assert!(!line_contains_normalized(&upgrade, "更新日期: 2026-02-09"));
    assert!(!contains_normalized(&upgrade, "Runtime 默认**关闭**"));
    assert!(!contains_normalized(
        &upgrade,
        "runtime.enabled=false (默认)"
    ));
    assert!(!line_contains_normalized(
        &upgrade,
        "| jq | 推荐 | 推荐 | machine JSON smoke 使用 |"
    ));

    let matrix = read("docs/CLI_CONTRACT_MATRIX.md");
    assert!(contains_normalized(
        &matrix,
        "rust/crates/fusion-cli/tests/"
    ));
    assert!(contains_normalized(
        &matrix,
        "update this matrix together with `rust/crates/fusion-cli/tests/repo_contract.rs` and the relevant shell/CLI contract tests"
    ));
    assert!(contains_normalized(
        &matrix,
        "generated `.fusion/config.yaml` starts from the checked-in `templates/config.yaml` baseline"
    ));
    assert!(contains_normalized(
        &matrix,
        "writes host-local `.claude/settings.local.json`, not a repository contract artifact"
    ));
    assert!(contains_normalized(
        &matrix,
        "checked-in `.claude/settings.example.json` remains the manual wiring template"
    ));
    assert!(contains_normalized(
        &matrix,
        "cross-platform-smoke-summary.json"
    ));
    assert!(contains_normalized(
        &matrix,
        "/tmp/cross-platform-smoke-macos/cross-platform-smoke-summary.json"
    ));
    assert!(contains_normalized(
        &matrix,
        "/tmp/cross-platform-smoke-windows/cross-platform-smoke-summary.json"
    ));
    assert!(contains_normalized(
        &matrix,
        "remote-ci-evidence-summary.json"
    ));
    assert!(contains_normalized(
        &matrix,
        "remote promotion evidence payload"
    ));
    assert!(contains_normalized(&matrix, "promotion_ready"));
    assert!(contains_normalized(&matrix, "required_jobs"));
    assert!(contains_normalized(&matrix, "missing_jobs"));
    assert!(contains_normalized(&matrix, "failed_jobs"));
    assert!(contains_normalized(
        &matrix,
        "cross-platform smoke summary payload"
    ));
    assert!(contains_normalized(&matrix, "completed_commands_count"));
    assert!(contains_normalized(&matrix, "UNDERSTAND handoff"));
    assert!(contains_normalized(
        &matrix,
        "Current state: <status> @ <phase>"
    ));
    assert!(contains_normalized(&matrix, "Next action: <...>"));
    assert!(contains_normalized(&matrix, "reviewer-gate next action"));
    for marker in [
        "workflow_id",
        "goal",
        "understand_mode",
        "understand_forced",
        "understand_decision",
        "codex_session",
        "claude_session",
        "planner_codex_session",
        "planner_claude_session",
        "coder_codex_session",
        "coder_claude_session",
        "reviewer_codex_session",
        "reviewer_claude_session",
        "runtime_state",
        "guardian_status",
        "guardian_no_progress_rounds",
        "guardian_same_error_count",
        "guardian_wall_time_ms",
        "safe_backlog_last_added",
        "safe_backlog_last_injected_at_iso",
        "runtime_last_event_id",
        "runtime_scheduler_enabled",
        "agents_enabled",
        "agent_mode",
        "agent_explain_level",
        "agent_current_batch_id",
        "agent_active_roles",
        "agent_current_batch_tasks",
        "agent_review_queue",
        "agent_review_queue_size",
        "agent_last_decision_reason",
        "agent_batch_reason",
        "agent_collaboration_mode",
        "agent_turn_role",
        "agent_turn_task_id",
        "agent_turn_kind",
        "agent_pending_reviews",
        "agent_blocked_handoff_reason",
        "agent_selected_reasons",
        "agent_blocked_reasons",
        "agent_review_reasons",
    ] {
        assert!(
            contains_normalized(&matrix, marker),
            "docs/CLI_CONTRACT_MATRIX.md missing machine status marker: {marker}"
        );
    }

    let e2e = read("docs/E2E_EXAMPLE.md");
    assert!(contains_normalized(
        &e2e,
        "reflects the checked-in `templates/config.yaml` baseline"
    ));
    assert!(contains_normalized(
        &e2e,
        "UNDERSTAND runner currently minimal; proceed to INITIALIZE"
    ));
    assert!(contains_normalized(
        &e2e,
        "[fusion] Current state: in_progress @ INITIALIZE"
    ));
    assert!(contains_normalized(
        &e2e,
        "[fusion] Next action: Initialize workspace files and proceed to ANALYZE"
    ));
    assert!(contains_normalized(
        &e2e,
        "_runtime.understand.mode=minimal"
    ));
    assert!(!contains_normalized(&e2e, "score=8 >= 7"));
    assert!(contains_normalized(
        &e2e,
        "Actual runs consume the generated `.fusion/config.yaml` initialized from that baseline."
    ));
    assert!(contains_normalized(
        &e2e,
        "If maintainers change `backend_routing`, update this example together with the template."
    ));
    assert!(contains_normalized(
        &e2e,
        "update the affected active docs together with `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`"
    ));
    assert!(contains_normalized(
        &e2e,
        "checked-in `.claude/settings.example.json` file is just the template"
    ));
    assert!(contains_normalized(
        &e2e,
        "actual `.claude/settings.json` and `.claude/settings.local.json` files are host-local hook configuration"
    ));

    let compatibility = read("docs/COMPATIBILITY.md");
    assert!(contains_normalized(
        &compatibility,
        "bash scripts/ci-cross-platform-smoke.sh"
    ));
    assert!(contains_normalized(
        &compatibility,
        "cargo test --release -p fusion-cli --test repo_contract"
    ));
    assert!(contains_normalized(
        &compatibility,
        "cargo test --release -p fusion-cli --test shell_contract"
    ));
    assert!(contains_normalized(
        &compatibility,
        "/tmp/cross-platform-smoke-macos/cross-platform-smoke-summary.json"
    ));
    assert!(contains_normalized(
        &compatibility,
        "/tmp/cross-platform-smoke-windows/cross-platform-smoke-summary.json"
    ));
    assert!(contains_normalized(
        &compatibility,
        "bash scripts/ci-cross-platform-json-smoke.sh /tmp/cross-platform-smoke-macos/cross-platform-smoke-summary.json"
    ));
    assert!(contains_normalized(
        &compatibility,
        "bash scripts/ci-cross-platform-json-smoke.sh /tmp/cross-platform-smoke-windows/cross-platform-smoke-summary.json"
    ));
    assert!(contains_normalized(
        &compatibility,
        "bash scripts/ci-remote-evidence.sh --repo dtamade/fafafa-skills-fusion --branch main --json"
    ));
    assert!(contains_normalized(
        &compatibility,
        "旧 runtime/reference 层已从仓库移除"
    ));
    assert!(contains_normalized(
        &compatibility,
        "旧解释器探测与旧 runtime/reference 路径都已退出仓库"
    ));
    assert!(contains_normalized(
        &compatibility,
        "当前 live 配置文档不再公开多 runtime engine 选择"
    ));
    assert!(contains_normalized(
        &compatibility,
        "请同步更新相关活文档，以及 `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`"
    ));
    assert!(contains_normalized(
        &compatibility,
        "`jq` 不是运行时必需依赖"
    ));
    assert!(!compatibility.contains("`grep -A`"));
    assert!(!compatibility.contains("`jq` JSON 处理"));
    assert!(!compatibility.contains("推荐依赖"));

    let roadmap = read("docs/RUST_FUSION_BRIDGE_ROADMAP.md");
    assert!(contains_normalized(
        &roadmap,
        "旧 runtime/reference 文件已从仓库移除"
    ));
    assert!(contains_normalized(&roadmap, "Rust `repo_contract`"));
    assert!(contains_normalized(&roadmap, "Rust `shell_contract`"));
    assert!(contains_normalized(
        &roadmap,
        "不能把这里的 crate 名直接当成用户入口"
    ));
    assert!(contains_normalized(
        &roadmap,
        "当前用户可见入口仍以 `fusion-start.sh`、`fusion-status.sh`、`fusion-resume.sh`、`fusion-codeagent.sh`"
    ));
    for needle in [
        "`scripts/fusion-start.sh`",
        "`fusion-bridge start`",
        "`scripts/fusion-resume.sh`",
        "`fusion-bridge resume`",
        "`scripts/fusion-status.sh`",
        "`fusion-bridge status`",
        "`scripts/fusion-codeagent.sh`",
        "`fusion-bridge codeagent`",
        "`scripts/fusion-pretool.sh`",
        "`fusion-bridge hook pretool`",
        "`scripts/fusion-posttool.sh`",
        "`fusion-bridge hook posttool`",
        "`scripts/fusion-stop-guard.sh`",
        "`fusion-bridge hook stop-guard`",
    ] {
        assert!(contains_normalized(&roadmap, needle));
    }
    assert!(contains_normalized(
        &roadmap,
        "请同步更新相关活文档，以及 `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`"
    ));
    assert!(contains_normalized(
        &roadmap,
        "当前 live 配置文档已不再公开多 engine 选择"
    ));
    assert!(contains_normalized(
        &roadmap,
        "当前主线文档已不再把其他 engine 作为 live 选项"
    ));
    assert!(contains_normalized(
        &roadmap,
        "`docs/V3_GA_EXECUTION_ROADMAP.md`"
    ));
    assert!(contains_normalized(
        &roadmap,
        "当前 live 执行顺序与发布收口应以 `docs/V3_GA_EXECUTION_ROADMAP.md` 为准"
    ));
    assert!(contains_normalized(
        &roadmap,
        "当前应理解为 Rust 主线配置能稳定完成端到端小目标流程"
    ));
    assert!(contains_normalized(
        &roadmap,
        "`.fusion/config.yaml`（工作区生成配置；由受版本控制的 `templates/config.yaml` 初始化，不是模板真源）"
    ));
    assert!(contains_normalized(
        &roadmap,
        "`templates/config.yaml`（受版本控制基线；供 `fusion-bridge init` / `scripts/fusion-init.sh` 生成 `.fusion/config.yaml`）"
    ));
    assert!(!roadmap.contains("fusion-cli status"));

    let legacy_roadmap = read("ROADMAP.md");
    assert!(contains_normalized(
        &legacy_roadmap,
        "`fusion-stop-guard.sh` → thin wrapper / `fusion-bridge hook stop-guard`"
    ));
    assert!(contains_normalized(
        &legacy_roadmap,
        "`fusion-pretool.sh` → thin wrapper / hook pretool 入口"
    ));
    assert!(contains_normalized(
        &legacy_roadmap,
        "`fusion-posttool.sh` → thin wrapper / hook posttool 入口"
    ));
    assert!(contains_normalized(
        &legacy_roadmap,
        "`docs/V3_GA_EXECUTION_ROADMAP.md`"
    ));
    assert!(contains_normalized(
        &legacy_roadmap,
        "当前 live 执行顺序与 GA 收口请以 `docs/V3_GA_EXECUTION_ROADMAP.md` 为准"
    ));

    let summary = read("docs/REPO_CONVERGENCE_SUMMARY_2026-03.md");
    assert!(contains_normalized(
        &summary,
        "Former runtime/reference layer has been removed from the repository"
    ));
    assert!(contains_normalized(
        &summary,
        "bash scripts/ci-machine-mode-smoke.sh"
    ));
    assert!(contains_normalized(&summary, "cargo test --release"));
    for doc_ref in [
        "`docs/UPGRADE_v2_COMPAT.md`",
        "`docs/COMPATIBILITY.md`",
        "`docs/CLI_CONTRACT_MATRIX.md`",
        "`PARALLEL_EXECUTION.md`",
        "`CONTRIBUTING.md`",
        "`CONTRIBUTING.zh-CN.md`",
    ] {
        assert!(contains_normalized(&summary, doc_ref));
    }
    assert!(contains_normalized(
        &summary,
        "`jq` is optional for machine JSON smoke or manual inspection"
    ));
    assert!(contains_normalized(
        &summary,
        "update the affected active docs together with `rust/crates/fusion-cli/tests/repo_contract.rs`"
    ));
    assert!(contains_normalized(
        &summary,
        "Generated local Rust caches such as `rust/target/` and `rust/.cargo-codex/` stay ignored"
    ));
    assert!(contains_normalized(
        &summary,
        "Host-local tool settings such as `.ace-tool/`, `.claude/settings.json`, and `.claude/settings.local.json` are local machine state"
    ));
    assert!(contains_normalized(
        &summary,
        "`.claude/settings.example.json` remains the checked-in template"
    ));

    let ga_roadmap = read("docs/V3_GA_EXECUTION_ROADMAP.md");
    assert!(contains_normalized(
        &ga_roadmap,
        "current execution source of truth"
    ));
    assert!(contains_normalized(
        &ga_roadmap,
        "v2.6 convergence complete, before v3.0 GA"
    ));
    assert!(contains_normalized(
        &ga_roadmap,
        "Rust / `fusion-bridge` is already the primary control plane"
    ));
    assert!(contains_normalized(&ga_roadmap, "minimum GA scope"));
    assert!(contains_normalized(
        &ga_roadmap,
        "do not add `/fusion explain` or dual-model collaboration to the current GA batch"
    ));
    assert!(contains_normalized(
        &ga_roadmap,
        "`docs/CLI_CONTRACT_MATRIX.md`"
    ));
    assert!(contains_normalized(&ga_roadmap, "cargo test --release"));
    assert!(contains_normalized(&ga_roadmap, "macOS"));
    assert!(contains_normalized(&ga_roadmap, "Windows (Git Bash)"));
    assert!(contains_normalized(
        &ga_roadmap,
        "smoke jobs are already wired, but active docs should keep partial-verification wording until fresh CI evidence upgrades that status"
    ));
    assert!(contains_normalized(&ga_roadmap, "ci-remote-evidence.sh"));
    assert!(contains_normalized(
        &ga_roadmap,
        "WSL remains post-GA evidence and is not a current GA blocker"
    ));

    let protocol = read("EXECUTION_PROTOCOL.md");
    assert!(contains_normalized(&protocol, "cargo test --release"));
    assert!(contains_normalized(
        &protocol,
        "Run the project-appropriate verification command"
    ));
    assert!(contains_normalized(&protocol, "这里只描述 provider 层调用"));
    assert!(contains_normalized(&protocol, "控制面入口仍是 `/fusion`"));
    assert!(contains_normalized(&protocol, "不是用户恢复入口"));
    assert!(contains_normalized(
        &protocol,
        "用户恢复入口仍是 `/fusion resume`"
    ));
    assert!(contains_normalized(
        &protocol,
        "`.fusion/config.yaml` - 由 `templates/config.yaml` 生成的工作区配置"
    ));
    assert!(contains_normalized(
        &protocol,
        "请同步更新相关活文档，以及 `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`"
    ));

    let skill = read("SKILL.md");
    assert!(contains_normalized(
        &skill,
        "`.claude/settings.example.json` 只是受版本控制模板"
    ));
    assert!(contains_normalized(
        &skill,
        "`.claude/settings.json` 与 `.claude/settings.local.json` 属于宿主本地 Hook 配置"
    ));
    assert!(contains_normalized(
        &skill,
        "请同步更新相关活文档，以及 `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`"
    ));

    let recovery = read("SESSION_RECOVERY.md");
    assert!(contains_normalized(&recovery, "仓库已移除的旧恢复实现"));
    assert!(contains_normalized(&recovery, "当成用户恢复入口"));
    assert!(contains_normalized(&recovery, "`/fusion resume`"));
    assert!(contains_normalized(
        &recovery,
        "Current state: <status> @ <phase>"
    ));
    assert!(contains_normalized(&recovery, "Next action: <...>"));
    assert!(contains_normalized(
        &recovery,
        "`.claude/settings.example.json` 模板 + 宿主本地 `.claude/settings.json` / `.claude/settings.local.json`"
    ));
    assert!(contains_normalized(
        &recovery,
        "`.claude/settings.example.json` 是受版本控制的模板"
    ));
    assert!(contains_normalized(
        &recovery,
        "请同步更新相关活文档，以及 `rust/crates/fusion-cli/tests/repo_contract.rs` / `rust/crates/fusion-cli/tests/shell_contract.rs`"
    ));

    let root_example = read("examples/root-session/README.md");
    assert!(contains_normalized(
        &root_example,
        "This repository does **not** keep live runtime session artifacts at the repository root."
    ));
    assert!(contains_normalized(
        &root_example,
        "`.fusion/config.yaml` is generated workspace state initialized from the checked-in `templates/config.yaml` baseline."
    ));

    let kernel_design = read("docs/RUNTIME_KERNEL_DESIGN.md");
    assert!(contains_normalized(
        &kernel_design,
        "历史 runtime kernel 的设计基线"
    ));
    assert!(contains_normalized(
        &kernel_design,
        "legacy compat adapter 已移除"
    ));
}

#[test]
fn repo_hygiene_docs_and_templates_match_rust_only_repo() {
    let root = repo_root();

    for file in ["findings.md", "progress.md", "task_plan.md"] {
        assert!(
            !root.join(file).exists(),
            "{file} should live under .fusion/ or templates/, not repo root"
        );
    }

    assert!(root.join("examples/root-session/README.md").is_file());
    assert!(root.join(".claude/settings.example.json").is_file());

    let gitignore = read(".gitignore");
    let retired_cache = retired_cache_dir();
    let retired_lang = retired_lang_word();
    let retired_version = retired_version_label();
    let retired_version_field_name = retired_version_field();
    let retired_module = retired_module_invocation();
    let retired_test_cmd = retired_test_command();
    let retired_short_env = retired_short_env_word();
    let retired_isolated_env = retired_isolated_env_word();
    for ignore_line in [
        "/findings.md",
        "/progress.md",
        "/task_plan.md",
        ".ace-tool/",
        "rust/target/",
        "rust/.cargo-codex/",
        ".claude/settings.json",
        ".claude/settings.local.json",
    ] {
        assert!(line_equals_normalized(&gitignore, ignore_line));
    }
    assert!(!line_equals_normalized(
        &gitignore,
        ".claude/settings.example.json"
    ));
    assert!(!line_equals_normalized(
        &gitignore,
        &format!("{}/", retired_bytecode_cache_dir())
    ));
    assert!(!line_equals_normalized(&gitignore, &retired_cache));

    let hygiene = read("docs/REPO_HYGIENE.md");
    assert!(contains_normalized(&hygiene, ".fusion/"));
    assert!(contains_normalized(&hygiene, "examples/"));
    assert!(contains_normalized(&hygiene, "Rust build outputs"));
    assert!(contains_normalized(&hygiene, "host-local tool settings"));
    assert!(contains_normalized(
        &hygiene,
        "generated local Rust caches such as `rust/.cargo-codex/`"
    ));
    assert!(contains_normalized(
        &hygiene,
        "`rust/.cargo-codex/`: local crate cache populated by Codex-side cargo workflows"
    ));
    assert!(contains_normalized(
        &hygiene,
        "`.ace-tool/`: local tool workspace/cache data"
    ));
    assert!(contains_normalized(
        &hygiene,
        "`.claude/settings.json`: host-local hook wiring/configuration file"
    ));
    assert!(contains_normalized(
        &hygiene,
        "`.claude/settings.local.json`: host-local override file written by doctor/fix flows"
    ));
    assert!(contains_normalized(
        &hygiene,
        "`.claude/settings.example.json`: checked-in template used to generate host-local hook configuration"
    ));
    assert!(contains_normalized(
        &hygiene,
        "Only the example template is intended to stay tracked."
    ));
    assert!(contains_normalized(
        &hygiene,
        "Confirm any repository/runtime contract change is reflected in the active docs and `rust/crates/fusion-cli/tests/repo_contract.rs`"
    ));
    assert!(!hygiene.contains(&retired_cache));

    let contributing = read("CONTRIBUTING.md");
    let contributing_lower = contributing.to_lowercase();
    assert!(contains_normalized(&contributing, "cargo test --release"));
    assert!(contains_normalized(&contributing, "Rust stable toolchain"));
    assert!(contains_normalized(
        &contributing,
        "optional `jq` for machine JSON smoke or manual JSON inspection"
    ));
    assert!(contains_normalized(
        &contributing,
        "Treat `rust/target/` and `rust/.cargo-codex/` as generated local Rust caches"
    ));
    assert!(contains_normalized(
        &contributing,
        "Treat `.ace-tool/`, `.claude/settings.json`, and `.claude/settings.local.json` as host-local tool state"
    ));
    assert!(contains_normalized(
        &contributing,
        "only `.claude/settings.example.json` remains the checked-in template"
    ));
    assert!(contains_normalized(
        &contributing,
        "If active docs or repository/runtime contracts changed, update `rust/crates/fusion-cli/tests/repo_contract.rs` too"
    ));
    assert!(!contributing_lower.contains(&retired_lang));
    assert!(!contributing_lower.contains(&retired_module));
    assert!(!contributing_lower.contains(&retired_short_env));
    assert!(!contributing_lower.contains(&retired_isolated_env));
    assert!(!contributing.contains(&retired_version));
    assert!(!contributing.contains(&retired_test_cmd));
    assert!(!contributing.contains(&retired_version_field_name));

    let contributing_zh = read("CONTRIBUTING.zh-CN.md");
    let contributing_zh_lower = contributing_zh.to_lowercase();
    assert!(contains_normalized(
        &contributing_zh,
        "cargo test --release"
    ));
    assert!(contains_normalized(
        &contributing_zh,
        "Rust stable toolchain"
    ));
    assert!(contains_normalized(
        &contributing_zh,
        "可选的 `jq`，仅用于 machine JSON smoke 或人工 JSON 检查"
    ));
    assert!(contains_normalized(
        &contributing_zh,
        "`rust/target/`、`rust/.cargo-codex/` 这类本地 Rust cache 属于机器生成状态"
    ));
    assert!(contains_normalized(
        &contributing_zh,
        "`.ace-tool/`、`.claude/settings.json`、`.claude/settings.local.json` 属于宿主本地工具状态"
    ));
    assert!(contains_normalized(
        &contributing_zh,
        "只有 `.claude/settings.example.json` 保持为受版本控制的模板"
    ));
    assert!(contains_normalized(
        &contributing_zh,
        "若活文档或仓库/runtime 契约发生变化，已同步更新 `rust/crates/fusion-cli/tests/repo_contract.rs`"
    ));
    assert!(!contributing_zh_lower.contains(&retired_lang));
    assert!(!contributing_zh_lower.contains(&retired_module));
    assert!(!contributing_zh_lower.contains(&retired_short_env));
    assert!(!contributing_zh_lower.contains(&retired_isolated_env));
    assert!(!contributing_zh.contains(&retired_test_cmd));
    assert!(!contributing_zh.contains(&retired_version));

    let pr_template = read(".github/PULL_REQUEST_TEMPLATE.md");
    let pr_template_lower = pr_template.to_lowercase();
    assert!(contains_normalized(
        &pr_template,
        "cd rust && cargo test --release"
    ));
    assert!(contains_normalized(
        &pr_template,
        "`rust/crates/fusion-cli/tests/repo_contract.rs` was updated too"
    ));
    assert!(contains_normalized(
        &pr_template,
        "`.fusion/config.yaml` is still treated as generated workspace state from `templates/config.yaml`"
    ));
    assert!(contains_normalized(
        &pr_template,
        "`.claude/settings.example.json` remains the checked-in template and `.claude/settings.json` / `.claude/settings.local.json` remain host-local files"
    ));
    assert!(!pr_template_lower.contains(&retired_lang));
    assert!(!pr_template_lower.contains(&retired_module));
    assert!(!pr_template_lower.contains(&retired_short_env));
    assert!(!pr_template_lower.contains(&retired_isolated_env));
    assert!(!pr_template.contains(&retired_test_cmd));

    let bug_template = read(".github/ISSUE_TEMPLATE/bug_report.md");
    let bug_template_lower = bug_template.to_lowercase();
    assert!(contains_normalized(&bug_template, "fusion-bridge"));
    assert!(contains_normalized(
        &bug_template,
        "fresh `.fusion/config.yaml` generated from `templates/config.yaml`"
    ));
    assert!(contains_normalized(
        &bug_template,
        "Hook config in use (`.claude/settings.json` / `.claude/settings.local.json` / none)"
    ));
    assert!(contains_normalized(
        &bug_template,
        "checked-in `.claude/settings.example.json` template was copied into a host-local file"
    ));
    assert!(!bug_template_lower.contains(&retired_lang));
    assert!(!bug_template_lower.contains(&retired_module));
    assert!(!bug_template_lower.contains(&retired_short_env));
    assert!(!bug_template_lower.contains(&retired_isolated_env));
    assert!(!bug_template.contains(&retired_version_field_name));

    let feature_template = read(".github/ISSUE_TEMPLATE/feature_request.md");
    let feature_template_lower = feature_template.to_lowercase();
    assert!(contains_normalized(
        &feature_template,
        "generated `.fusion/config.yaml` workspace contract"
    ));
    assert!(contains_normalized(
        &feature_template,
        "checked-in `templates/config.yaml` baseline"
    ));
    assert!(contains_normalized(
        &feature_template,
        "`.claude/settings.example.json`, or any host-local `.claude/settings.json` / `.claude/settings.local.json` behavior"
    ));
    assert!(contains_normalized(
        &feature_template,
        "`rust/crates/fusion-cli/tests/repo_contract.rs` or other release contract gates change together"
    ));
    assert!(!feature_template_lower.contains(&retired_lang));
    assert!(!feature_template_lower.contains(&retired_module));
    assert!(!feature_template_lower.contains(&retired_short_env));
    assert!(!feature_template_lower.contains(&retired_isolated_env));
    assert!(!feature_template.contains(&retired_version_field_name));
}

#[test]
fn ci_contract_gate_stays_release_rust_only() {
    let workflow = read(".github/workflows/ci-contract-gates.yml");
    let workflow_lower = workflow.to_lowercase();
    let retired_test_cmd = retired_test_command();
    let retired_setup = retired_setup_action();
    let retired_install = retired_install_step();
    let retired_module = retired_module_invocation();
    let retired_skip = retired_skip_flag();
    let retired_short_env = retired_short_env_word();
    let retired_isolated_env = retired_isolated_env_word();
    assert!(!workflow_lower.contains(&retired_lang_word()));
    assert!(!workflow.contains(&retired_setup));
    assert!(!workflow.contains(&retired_install));
    assert!(!workflow.contains(&retired_test_cmd));
    assert!(!workflow_lower.contains(&retired_module));
    assert!(!workflow_lower.contains(&retired_short_env));
    assert!(!workflow_lower.contains(&retired_isolated_env));
    assert!(contains_normalized(&workflow, "bash -n scripts/*.sh"));
    assert!(contains_normalized(
        &workflow,
        "bash scripts/ci-machine-mode-smoke.sh"
    ));
    assert!(contains_normalized(
        &workflow,
        "bash scripts/ci-cross-platform-smoke.sh"
    ));
    assert!(contains_normalized(
        &workflow,
        "cargo clippy --release --workspace --all-targets -- -D warnings"
    ));
    assert!(contains_normalized(&workflow, "cargo test --release"));
    assert!(contains_normalized(&workflow, "cargo fmt --all -- --check"));
    assert!(line_equals_normalized(
        &workflow,
        "uses: actions/upload-artifact@v4"
    ));
    for artifact_path in [
        "/tmp/release-audit-dry-run.json",
        "/tmp/runner-suites.json",
        "/tmp/runner-contract.json",
    ] {
        assert!(line_equals_normalized(&workflow, artifact_path));
    }

    let contract_gates = job_block(
        &workflow,
        "  contract-gates:\n",
        Some("\n  cross-platform-smoke-macos:\n"),
    );
    assert!(contains_normalized(
        contract_gates,
        "cargo build --release -p fusion-cli --bin fusion-bridge"
    ));
    assert!(appears_before_normalized(
        contract_gates,
        "cargo build --release -p fusion-cli --bin fusion-bridge",
        "- name: Machine mode smoke gate"
    ));

    let macos = job_block(
        &workflow,
        "  cross-platform-smoke-macos:\n",
        Some("\n  cross-platform-smoke-windows:\n"),
    );
    assert!(line_equals_normalized(macos, "runs-on: macos-latest"));
    assert!(contains_normalized(
        macos,
        "cargo test --release -p fusion-cli --test cli_smoke"
    ));
    assert!(contains_normalized(
        macos,
        "bash scripts/ci-cross-platform-smoke.sh --artifacts-dir /tmp/cross-platform-smoke-macos --platform-label macos"
    ));
    assert!(contains_normalized(
        macos,
        "bash scripts/ci-cross-platform-json-smoke.sh /tmp/cross-platform-smoke-macos/cross-platform-smoke-summary.json"
    ));
    assert!(contains_normalized(
        macos,
        "name: cross-platform-smoke-macos-json"
    ));
    assert!(contains_normalized(
        macos,
        "/tmp/cross-platform-smoke-macos/cross-platform-smoke-summary.json"
    ));
    assert!(appears_before_normalized(
        macos,
        "cargo build --release -p fusion-cli --bin fusion-bridge",
        "- name: Cross-platform shell smoke"
    ));

    let windows = job_block(&workflow, "  cross-platform-smoke-windows:\n", None);
    assert!(line_equals_normalized(windows, "runs-on: windows-latest"));
    assert!(line_equals_normalized(windows, "shell: bash"));
    assert!(contains_normalized(
        windows,
        "cargo test --release -p fusion-cli --test cli_smoke"
    ));
    assert!(contains_normalized(
        windows,
        "bash scripts/ci-cross-platform-smoke.sh --artifacts-dir /tmp/cross-platform-smoke-windows --platform-label windows-git-bash"
    ));
    assert!(contains_normalized(
        windows,
        "bash scripts/ci-cross-platform-json-smoke.sh /tmp/cross-platform-smoke-windows/cross-platform-smoke-summary.json"
    ));
    assert!(contains_normalized(
        windows,
        "name: cross-platform-smoke-windows-json"
    ));
    assert!(contains_normalized(
        windows,
        "/tmp/cross-platform-smoke-windows/cross-platform-smoke-summary.json"
    ));
    assert!(appears_before_normalized(
        windows,
        "cargo build --release -p fusion-cli --bin fusion-bridge",
        "- name: Cross-platform shell smoke"
    ));

    let machine_mode = read("scripts/ci-machine-mode-smoke.sh");
    let machine_mode_lower = machine_mode.to_lowercase();
    assert!(contains_normalized(
        &machine_mode,
        "release-contract-audit.sh"
    ));
    assert!(contains_normalized(
        &machine_mode,
        "--dry-run --json --fast --skip-rust"
    ));
    assert!(contains_normalized(
        &machine_mode,
        "regression --list-suites --json"
    ));
    assert!(contains_normalized(
        &machine_mode,
        "regression --suite contract --json --min-pass-rate 0.99"
    ));
    assert!(!machine_mode.contains(&retired_skip));
    assert!(!machine_mode_lower.contains(&retired_lang_word()));
    assert!(!machine_mode_lower.contains(&retired_module));
    assert!(!machine_mode_lower.contains(&retired_short_env));
    assert!(!machine_mode_lower.contains(&retired_isolated_env));

    let machine_json = read("scripts/ci-machine-json-smoke.sh");
    let machine_json_lower = machine_json.to_lowercase();
    for marker in [
        "schema_version",
        "step_rate_basis",
        "command_rate_basis",
        "rate_basis",
        "longest_scenario",
        "fastest_scenario",
        "scenario_count_by_result",
        "duration_stats",
        "failed_rate",
        "success_rate",
        "success_count",
        "failure_count",
        "total_scenarios",
    ] {
        assert!(
            machine_json.contains(marker),
            "ci-machine-json-smoke.sh missing marker: {marker}"
        );
    }
    assert!(!machine_json_lower.contains(&retired_lang_word()));
    assert!(!machine_json_lower.contains(&retired_module));
    assert!(!machine_json_lower.contains(&retired_short_env));
    assert!(!machine_json_lower.contains(&retired_isolated_env));

    let cross_platform = read("scripts/ci-cross-platform-smoke.sh");
    let cross_platform_lower = cross_platform.to_lowercase();
    assert!(contains_normalized(
        &cross_platform,
        "--artifacts-dir <path>"
    ));
    assert!(contains_normalized(
        &cross_platform,
        "--platform-label <label>"
    ));
    assert!(contains_normalized(
        &cross_platform,
        "cross-platform-smoke-summary.json"
    ));
    let cross_platform_json = read("scripts/ci-cross-platform-json-smoke.sh");
    let cross_platform_json_lower = cross_platform_json.to_lowercase();
    assert!(contains_normalized(
        &cross_platform_json,
        "Usage: ci-cross-platform-json-smoke.sh"
    ));
    assert!(contains_normalized(
        &cross_platform_json,
        "cross-platform-smoke-summary.json"
    ));
    assert!(contains_normalized(
        &cross_platform_json,
        "completed_commands_count"
    ));
    assert!(!cross_platform_json_lower.contains(&retired_lang_word()));
    assert!(!cross_platform_json_lower.contains(&retired_module));
    assert!(!cross_platform_json_lower.contains(&retired_short_env));
    assert!(!cross_platform_json_lower.contains(&retired_isolated_env));
    let remote_evidence = read("scripts/ci-remote-evidence.sh");
    let remote_evidence_lower = remote_evidence.to_lowercase();
    assert!(contains_normalized(
        &remote_evidence,
        "Usage: ci-remote-evidence.sh"
    ));
    assert!(contains_normalized(
        &remote_evidence,
        "remote-ci-evidence-summary.json"
    ));
    assert!(contains_normalized(&remote_evidence, "promotion_ready"));
    assert!(contains_normalized(&remote_evidence, "workflow_not_found"));
    assert!(!remote_evidence_lower.contains(&retired_lang_word()));
    assert!(!remote_evidence_lower.contains(&retired_module));
    assert!(!remote_evidence_lower.contains(&retired_short_env));
    assert!(!remote_evidence_lower.contains(&retired_isolated_env));
    assert!(!cross_platform_lower.contains(&retired_lang_word()));
    assert!(!cross_platform_lower.contains(&retired_module));
    assert!(!cross_platform_lower.contains(&retired_short_env));
    assert!(!cross_platform_lower.contains(&retired_isolated_env));
}

#[test]
fn safe_backlog_targets_rust_contract_tests() {
    let content = read("rust/crates/fusion-cli/src/safe_backlog_support.rs");
    assert!(contains_normalized(
        &content,
        "project_root.join(\"rust/crates/fusion-cli/tests\")"
    ));
    assert!(contains_normalized(&content, "补充 Rust 契约测试清单"));
    assert!(contains_normalized(
        &content,
        "output: \"rust/crates/fusion-cli/tests\".to_string()"
    ));
    assert!(!content.contains("tests/runtime"));
    assert!(!content.contains("scripts/runtime"));
}

#[test]
fn contract_tests_avoid_raw_stdio_contains_matchers() {
    let stdout_prefix = [".std".to_string(), "out(".to_string()].concat();
    let raw_contains = [stdout_prefix.clone(), "contains(".to_string()].concat();
    let exact_stdout_literal = [stdout_prefix.clone(), "\"".to_string()].concat();
    let predicate_contains = [
        stdout_prefix.clone(),
        "predicate".to_string(),
        "::str::contains(".to_string(),
    ]
    .concat();
    let predicates_contains = [
        stdout_prefix,
        "predicates".to_string(),
        "::str::contains(".to_string(),
    ]
    .concat();
    let stderr_prefix = [".std".to_string(), "err(".to_string()].concat();
    let raw_stderr_contains = [stderr_prefix.clone(), "contains(".to_string()].concat();
    let exact_stderr_literal = [stderr_prefix.clone(), "\"".to_string()].concat();
    let stderr_predicate_contains = [
        stderr_prefix.clone(),
        "predicate".to_string(),
        "::str::contains(".to_string(),
    ]
    .concat();
    let stderr_predicates_contains = [
        stderr_prefix,
        "predicates".to_string(),
        "::str::contains(".to_string(),
    ]
    .concat();

    for relative in [
        "rust/crates/fusion-cli/tests/cli_smoke.rs",
        "rust/crates/fusion-cli/tests/repo_contract.rs",
        "rust/crates/fusion-cli/tests/shell_contract.rs",
    ] {
        let content = read(relative);
        assert!(
            !content.contains(&raw_contains),
            "{relative} should avoid raw stdout contains matchers; prefer parsed output or normalized line assertions"
        );
        assert!(
            !content.contains(&exact_stdout_literal),
            "{relative} should avoid exact stdout string literals; prefer parsed output or normalized line assertions"
        );
        assert!(
            !content.contains(&predicate_contains),
            "{relative} should avoid predicate::str::contains on stdout; prefer parsed output or normalized line assertions"
        );
        assert!(
            !content.contains(&predicates_contains),
            "{relative} should avoid predicates::str::contains on stdout; prefer parsed output or normalized line assertions"
        );
        assert!(
            !content.contains(&raw_stderr_contains),
            "{relative} should avoid raw stderr contains matchers; prefer parsed output or normalized line assertions"
        );
        assert!(
            !content.contains(&exact_stderr_literal),
            "{relative} should avoid exact stderr string literals; prefer parsed output or normalized line assertions"
        );
        assert!(
            !content.contains(&stderr_predicate_contains),
            "{relative} should avoid predicate::str::contains on stderr; prefer parsed output or normalized line assertions"
        );
        assert!(
            !content.contains(&stderr_predicates_contains),
            "{relative} should avoid predicates::str::contains on stderr; prefer parsed output or normalized line assertions"
        );
    }
}

#[test]
fn cli_and_shell_contract_tests_only_use_whitelisted_raw_contains_calls() {
    let allowed_fragments = [
        "normalize_whitespace(haystack).contains(&normalize_whitespace(needle))",
        "normalize_whitespace(line).contains(&normalize_whitespace(needle))",
        "command.contains(&retired_test)",
        "command.contains(\"test_fusion_\")",
    ];

    for relative in [
        "rust/crates/fusion-cli/tests/cli_smoke.rs",
        "rust/crates/fusion-cli/tests/shell_contract.rs",
    ] {
        let content = read(relative);
        for (line_no, line) in content.lines().enumerate() {
            if line.contains(".contains(")
                && !allowed_fragments
                    .iter()
                    .any(|fragment| line.contains(fragment))
            {
                panic!(
                    "{relative}:{} should avoid raw .contains(...) outside helper internals and command/token checks: {}",
                    line_no + 1,
                    line.trim()
                );
            }
        }
    }
}

#[test]
fn cli_smoke_only_uses_whitelisted_business_contains_normalized_calls() {
    let allowed_fragments = [
        "fn contains_normalized(haystack: &str, needle: &str) -> bool {",
        "contains_normalized(&stderr, expected)",
    ];

    let content = read("rust/crates/fusion-cli/tests/cli_smoke.rs");
    for (line_no, line) in content.lines().enumerate() {
        if line.contains("contains_normalized(")
            && !line.contains("line_contains_normalized(")
            && !line.contains("assert_stderr_contains_normalized(")
            && !allowed_fragments
                .iter()
                .any(|fragment| line.contains(fragment))
        {
            panic!(
                "rust/crates/fusion-cli/tests/cli_smoke.rs:{} should avoid non-whitelisted contains_normalized(...) usage: {}",
                line_no + 1,
                line.trim()
            );
        }
    }
}

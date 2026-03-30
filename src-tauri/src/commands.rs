use crate::config::{ConfigProfile, HintCategory};
use crate::database::{SessionHistoryEntry, Snippet};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{command, AppHandle, Emitter, State};
use tokio::sync::mpsc;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTerminalRequest {
    pub label: String,
    pub working_directory: String,
    pub claude_args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub color_tag: Option<String>,
    pub nickname: Option<String>,
}

#[command]
pub async fn create_terminal(
    app: AppHandle,
    state: State<'_, AppState>,
    request: CreateTerminalRequest,
) -> Result<crate::terminal::TerminalConfig, String> {
    let (tx, mut rx) = mpsc::channel::<(String, Vec<u8>)>(100);

    // Compute log file path
    let log_path = {
        let data_dir = directories::ProjectDirs::from("com", "claudeterminal", "ClaudeTerminal")
            .ok_or("Failed to get project directories")?
            .data_dir()
            .to_path_buf();
        let logs_dir = data_dir.join("logs");
        std::fs::create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.log", uuid::Uuid::new_v4(), timestamp);
        logs_dir.join(filename).to_string_lossy().to_string()
    };

    let config = {
        let mut terminals = state.terminals.lock().await;
        terminals.create_terminal(
            request.label.clone(),
            request.working_directory,
            request.claude_args,
            request.env_vars,
            request.color_tag,
            request.nickname,
            tx,
            Some(log_path.clone()),
        )?
    };

    // Insert session history entry
    {
        let db = state.db.lock().await;
        if let Err(e) = db.insert_session_history(
            &config.id,
            &config.label,
            &config.created_at.to_rfc3339(),
            Some(&log_path),
        ) {
            eprintln!("Failed to insert session history: {}", e);
        }
    }

    let terminal_id = config.id.clone();
    let db_arc = state.db.clone();
    let terminals_arc = state.terminals.clone();

    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Some((id, data)) = rx.recv().await {
            if let Err(e) = app_clone.emit("terminal-output", serde_json::json!({
                "id": id,
                "data": data,
            })) {
                eprintln!("Failed to emit terminal-output: {}", e);
                break;
            }
        }

        // Terminal process exited — update status, session history, and notify frontend
        // Note: the terminal may have already been removed by close_terminal(), so ignore errors
        {
            if let Ok(mut manager) = tokio::time::timeout(
                std::time::Duration::from_secs(2),
                terminals_arc.lock(),
            ).await {
                let _ = manager.update_status(&terminal_id, crate::terminal::TerminalStatus::Stopped);
            }
        }
        {
            let db = db_arc.lock().await;
            if let Err(e) = db.update_session_ended(&terminal_id, &chrono::Utc::now().to_rfc3339()) {
                eprintln!("Failed to update session ended for {}: {}", terminal_id, e);
            }
        }

        if let Err(e) = app_clone.emit("terminal-finished", serde_json::json!({
            "id": terminal_id,
        })) {
            eprintln!("Failed to emit terminal-finished: {}", e);
        }
    });

    Ok(config)
}

/// Maximum size for a single write to terminal (64 KB)
const MAX_TERMINAL_WRITE_SIZE: usize = 65_536;

#[command]
pub async fn write_to_terminal(
    state: State<'_, AppState>,
    id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    if data.len() > MAX_TERMINAL_WRITE_SIZE {
        return Err(format!(
            "Write payload too large ({} bytes). Maximum is {} bytes.",
            data.len(),
            MAX_TERMINAL_WRITE_SIZE
        ));
    }
    let mut terminals = state.terminals.lock().await;
    terminals.write(&id, &data)
}

#[command]
pub async fn resize_terminal(
    state: State<'_, AppState>,
    id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let mut terminals = state.terminals.lock().await;
    terminals.resize(&id, cols, rows)
}

#[command]
pub async fn close_terminal(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut terminals = state.terminals.lock().await;
    terminals.close(&id)
}

#[command]
pub async fn get_terminals(
    state: State<'_, AppState>,
) -> Result<Vec<crate::terminal::TerminalConfig>, String> {
    let terminals = state.terminals.lock().await;
    Ok(terminals.get_all_configs())
}

#[command]
pub async fn update_terminal_label(
    state: State<'_, AppState>,
    id: String,
    label: String,
) -> Result<(), String> {
    let mut terminals = state.terminals.lock().await;
    terminals.update_label(&id, label)
}

#[command]
pub async fn update_terminal_nickname(
    state: State<'_, AppState>,
    id: String,
    nickname: String,
) -> Result<(), String> {
    let mut terminals = state.terminals.lock().await;
    terminals.update_nickname(&id, nickname)
}

#[command]
pub async fn save_profile(
    state: State<'_, AppState>,
    profile: ConfigProfile,
) -> Result<(), String> {
    let db = state.db.lock().await;
    db.save_profile(&profile)
}

#[command]
pub async fn get_profiles(state: State<'_, AppState>) -> Result<Vec<ConfigProfile>, String> {
    let db = state.db.lock().await;
    db.get_profiles()
}

#[command]
pub async fn delete_profile(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().await;
    db.delete_profile(&id)
}

#[command]
pub async fn get_claude_version() -> Result<String, String> {
    let output = shell_command("claude", &["--version"])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
}

#[command]
pub async fn check_claude_update() -> Result<UpdateCheckResult, String> {
    // Get current version
    let current_output = shell_command("claude", &["--version"])
        .output()
        .map_err(|e| format!("Failed to get current version: {}", e))?;

    let current_version = String::from_utf8_lossy(&current_output.stdout)
        .trim()
        .to_string();

    if current_version.is_empty() {
        return Err("Claude Code is not installed".to_string());
    }

    // Get latest version from npm
    let npm_output = shell_command("npm", &["view", "@anthropic-ai/claude-code", "version"])
        .output()
        .map_err(|e| format!("Failed to check latest version: {}", e))?;

    let latest_version = String::from_utf8_lossy(&npm_output.stdout)
        .trim()
        .to_string();

    if latest_version.is_empty() {
        return Err("Failed to fetch latest version from npm".to_string());
    }

    // Extract version number from current version string (e.g., "1.0.17 (Claude Code)" -> "1.0.17")
    let current_ver_clean = current_version
        .split_whitespace()
        .next()
        .unwrap_or(&current_version)
        .to_string();

    let update_available = current_ver_clean != latest_version;

    Ok(UpdateCheckResult {
        current_version,
        latest_version,
        update_available,
    })
}

#[command]
pub async fn update_claude_code() -> Result<String, String> {
    let output = shell_command("npm", &["install", "-g", "@anthropic-ai/claude-code@latest"])
        .output()
        .map_err(|e| format!("Failed to run npm: {}", e))?;

    if output.status.success() {
        Ok("Claude Code updated successfully!".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!("{}{}", stderr, stdout))
    }
}

#[command]
pub fn get_hints() -> Vec<HintCategory> {
    crate::config::get_default_hints()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub node_installed: bool,
    pub node_version: Option<String>,
    pub npm_installed: bool,
    pub npm_version: Option<String>,
    pub claude_installed: bool,
    pub claude_version: Option<String>,
}

/// Shells that are allowed when reading `$SHELL` on non-Windows platforms.
const VALID_SHELLS: &[&str] = &[
    "/bin/bash",
    "/bin/sh",
    "/bin/zsh",
    "/bin/fish",
    "/bin/dash",
    "/usr/bin/bash",
    "/usr/bin/sh",
    "/usr/bin/zsh",
    "/usr/bin/fish",
    "/usr/bin/dash",
    "/usr/local/bin/bash",
    "/usr/local/bin/zsh",
    "/usr/local/bin/fish",
    "/opt/homebrew/bin/bash",
    "/opt/homebrew/bin/zsh",
    "/opt/homebrew/bin/fish",
];

/// Shell-escape a single argument by wrapping it in single quotes.
/// Any embedded single quotes are escaped as `'\''`.
fn shell_escape_arg(arg: &str) -> String {
    let mut escaped = String::with_capacity(arg.len() + 2);
    escaped.push('\'');
    for ch in arg.chars() {
        if ch == '\'' {
            escaped.push_str("'\\''");
        } else {
            escaped.push(ch);
        }
    }
    escaped.push('\'');
    escaped
}

/// Creates a Command that works cross-platform.
/// On Windows, wraps the command with `cmd /C` so that `.cmd`/`.bat` scripts
/// (like `npm.cmd`, `claude.cmd`) are resolved correctly.
fn shell_command(program: &str, args: &[&str]) -> std::process::Command {
    if cfg!(target_os = "windows") {
        let mut cmd = std::process::Command::new("cmd");
        cmd.arg("/C").arg(program);
        for arg in args {
            cmd.arg(arg);
        }
        // Prevent a console window from flashing on Windows
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        cmd
    } else {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        // Validate $SHELL against allowlist to prevent arbitrary binary execution
        let shell = if VALID_SHELLS.contains(&shell.as_str()) {
            shell
        } else {
            "/bin/bash".to_string()
        };
        let mut full_cmd = shell_escape_arg(program);
        for arg in args {
            full_cmd.push(' ');
            full_cmd.push_str(&shell_escape_arg(arg));
        }
        let mut cmd = std::process::Command::new(shell);
        cmd.arg("-lc").arg(&full_cmd);
        cmd
    }
}

#[command]
pub async fn check_system_requirements() -> Result<SystemStatus, String> {
    // Check Node.js
    let node_result = shell_command("node", &["--version"]).output();

    let (node_installed, node_version) = match node_result {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (true, Some(version))
        }
        _ => (false, None),
    };

    // Check npm
    let npm_result = shell_command("npm", &["--version"]).output();

    let (npm_installed, npm_version) = match npm_result {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (true, Some(version))
        }
        _ => (false, None),
    };

    // Check Claude Code
    let claude_result = shell_command("claude", &["--version"]).output();

    let (claude_installed, claude_version) = match claude_result {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (true, Some(version))
        }
        _ => (false, None),
    };

    Ok(SystemStatus {
        node_installed,
        node_version,
        npm_installed,
        npm_version,
        claude_installed,
        claude_version,
    })
}

#[command]
pub async fn install_claude_code() -> Result<String, String> {
    let output = shell_command("npm", &["install", "-g", "@anthropic-ai/claude-code"])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok("Claude Code installed successfully!".to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[command]
pub async fn send_notification(title: String, body: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        notify_rust::Notification::new()
            .summary(&title)
            .body(&body)
            .show()
            .map(|_| ())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn open_external_url(url: String) -> Result<(), String> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err("Only HTTP and HTTPS URLs are allowed".to_string());
    }
    open::that(&url).map_err(|e| e.to_string())
}

#[command]
pub async fn get_workspaces(
    state: State<'_, AppState>,
) -> Result<Vec<crate::database::WorkspaceInfo>, String> {
    let db = state.db.lock().await;
    db.get_workspaces()
}

#[command]
pub async fn delete_workspace(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    let db = state.db.lock().await;
    db.delete_workspace(&name)
}

#[command]
pub async fn save_workspace(
    state: State<'_, AppState>,
    name: String,
    terminals: Vec<crate::terminal::TerminalConfig>,
) -> Result<(), String> {
    let db = state.db.lock().await;
    db.save_workspace(&name, &terminals)
}

#[command]
pub async fn load_workspace(
    state: State<'_, AppState>,
    name: String,
) -> Result<Vec<crate::terminal::TerminalConfig>, String> {
    let db = state.db.lock().await;
    db.load_workspace(&name)
}

#[command]
pub async fn save_session_for_restore(state: State<'_, AppState>) -> Result<(), String> {
    let configs = {
        let terminals = state.terminals.lock().await;
        terminals.get_all_configs()
    };
    let db = state.db.lock().await;
    db.save_last_session(&configs)
}

#[command]
pub async fn get_last_session(
    state: State<'_, AppState>,
) -> Result<Option<Vec<crate::terminal::TerminalConfig>>, String> {
    let db = state.db.lock().await;
    db.load_last_session()
}

#[command]
pub async fn clear_last_session(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().await;
    db.clear_last_session()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileChangesResult {
    pub terminal_id: String,
    pub working_directory: String,
    pub changes: Vec<FileChange>,
    pub is_git_repo: bool,
    pub branch: Option<String>,
    pub error: Option<String>,
}

#[command]
pub async fn get_terminal_changes(
    state: State<'_, AppState>,
    id: String,
) -> Result<FileChangesResult, String> {
    let working_directory = {
        let terminals = state.terminals.lock().await;
        let configs = terminals.get_all_configs();
        configs
            .into_iter()
            .find(|c| c.id == id)
            .map(|c| c.working_directory.clone())
            .ok_or_else(|| "Terminal not found".to_string())?
    };

    // Check if it's a git repo and get branch name
    let branch_output = shell_command("git", &["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&working_directory)
        .output();

    let (is_git_repo, branch) = match branch_output {
        Ok(output) if output.status.success() => {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (true, Some(branch))
        }
        _ => (false, None),
    };

    if !is_git_repo {
        return Ok(FileChangesResult {
            terminal_id: id,
            working_directory,
            changes: vec![],
            is_git_repo: false,
            branch: None,
            error: None,
        });
    }

    // Get changed files
    let status_output = shell_command("git", &["status", "--porcelain"])
        .current_dir(&working_directory)
        .output()
        .map_err(|e| format!("Failed to run git status: {}", e))?;

    if !status_output.status.success() {
        return Ok(FileChangesResult {
            terminal_id: id,
            working_directory,
            changes: vec![],
            is_git_repo: true,
            branch,
            error: Some(String::from_utf8_lossy(&status_output.stderr).trim().to_string()),
        });
    }

    let stdout = String::from_utf8_lossy(&status_output.stdout);
    let changes: Vec<FileChange> = stdout
        .lines()
        .filter(|line| line.len() >= 3)
        .map(|line| {
            let code = &line[..2];
            let path = line[3..].to_string();
            let status = match code.trim() {
                "??" => "untracked",
                "A" | "A " => "new",
                "M" | "M " | " M" | "MM" => "modified",
                "D" | "D " | " D" => "deleted",
                r if r.starts_with('R') => "renamed",
                _ => "modified",
            };
            FileChange {
                path,
                status: status.to_string(),
            }
        })
        .collect();

    Ok(FileChangesResult {
        terminal_id: id,
        working_directory,
        changes,
        is_git_repo: true,
        branch,
        error: None,
    })
}

// ─── Git Worktree Commands ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: Option<String>,
    pub head_sha: String,
    pub is_main: bool,
    pub is_bare: bool,
    pub is_detached: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorktreeDetectResult {
    pub is_git_repo: bool,
    pub is_worktree: bool,
    pub main_repo_path: Option<String>,
    pub current_branch: Option<String>,
    pub worktree_root: Option<String>,
}

#[command]
pub async fn get_worktree_info(path: String) -> Result<WorktreeDetectResult, String> {
    // Check if inside a git work tree
    let inside_wt = shell_command("git", &["rev-parse", "--is-inside-work-tree"])
        .current_dir(&path)
        .output();

    let is_git_repo = matches!(inside_wt, Ok(ref o) if o.status.success()
        && String::from_utf8_lossy(&o.stdout).trim() == "true");

    if !is_git_repo {
        return Ok(WorktreeDetectResult {
            is_git_repo: false,
            is_worktree: false,
            main_repo_path: None,
            current_branch: None,
            worktree_root: None,
        });
    }

    // Get worktree root (--show-toplevel)
    let toplevel = shell_command("git", &["rev-parse", "--show-toplevel"])
        .current_dir(&path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    // Get git-dir and git-common-dir to detect if this is a linked worktree
    let git_dir = shell_command("git", &["rev-parse", "--git-dir"])
        .current_dir(&path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    let git_common_dir = shell_command("git", &["rev-parse", "--git-common-dir"])
        .current_dir(&path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    // If git-dir != git-common-dir, this is a linked worktree
    let is_worktree = match (&git_dir, &git_common_dir) {
        (Some(dir), Some(common)) => {
            let dir_canon = std::path::PathBuf::from(dir).canonicalize().ok();
            let common_canon = std::path::PathBuf::from(common).canonicalize().ok();
            match (dir_canon, common_canon) {
                (Some(d), Some(c)) => d != c,
                _ => dir != common,
            }
        }
        _ => false,
    };

    // Derive main repo path from git-common-dir (strip trailing .git)
    let main_repo_path = git_common_dir.and_then(|common| {
        let p = std::path::PathBuf::from(&common);
        let canonical = p.canonicalize().ok()?;
        // git-common-dir points to the .git directory; parent is the repo root
        if canonical.file_name().map(|f| f == ".git").unwrap_or(false) {
            canonical.parent().map(|p| p.to_string_lossy().to_string())
        } else {
            Some(canonical.to_string_lossy().to_string())
        }
    });

    // Get current branch
    let current_branch = shell_command("git", &["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            let b = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if b == "HEAD" { None } else { Some(b) }
        })
        .flatten();

    Ok(WorktreeDetectResult {
        is_git_repo: true,
        is_worktree,
        main_repo_path,
        current_branch,
        worktree_root: toplevel,
    })
}

#[command]
pub async fn list_worktrees(path: String) -> Result<Vec<WorktreeInfo>, String> {
    let output = shell_command("git", &["worktree", "list", "--porcelain"])
        .current_dir(&path)
        .output()
        .map_err(|e| format!("Failed to run git worktree list: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut is_first = true;

    // Parse porcelain output: blocks separated by blank lines
    for block in stdout.split("\n\n") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let mut wt_path = String::new();
        let mut head_sha = String::new();
        let mut branch: Option<String> = None;
        let mut is_bare = false;
        let mut is_detached = false;

        for line in block.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                wt_path = p.to_string();
            } else if let Some(h) = line.strip_prefix("HEAD ") {
                head_sha = h[..7.min(h.len())].to_string();
            } else if let Some(b) = line.strip_prefix("branch ") {
                // Strip refs/heads/ prefix
                branch = Some(
                    b.strip_prefix("refs/heads/")
                        .unwrap_or(b)
                        .to_string(),
                );
            } else if line == "bare" {
                is_bare = true;
            } else if line == "detached" {
                is_detached = true;
            }
        }

        if !wt_path.is_empty() {
            worktrees.push(WorktreeInfo {
                path: wt_path,
                branch,
                head_sha,
                is_main: is_first,
                is_bare,
                is_detached,
            });
        }
        is_first = false;
    }

    Ok(worktrees)
}

#[command]
pub async fn get_repo_branches(path: String) -> Result<Vec<String>, String> {
    let output = shell_command("git", &["branch", "--format=%(refname:short)"])
        .current_dir(&path)
        .output()
        .map_err(|e| format!("Failed to list branches: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    Ok(branches)
}

#[command]
pub async fn create_worktree(
    repo_path: String,
    worktree_path: String,
    branch: String,
    create_branch: bool,
) -> Result<WorktreeInfo, String> {
    // Validate branch name
    let branch_regex = regex::Regex::new(r"^[a-zA-Z0-9_./-]+$")
        .map_err(|e| e.to_string())?;
    if !branch_regex.is_match(&branch) {
        return Err("Invalid branch name. Use only letters, numbers, dots, hyphens, underscores, and slashes.".to_string());
    }

    let output = if create_branch {
        shell_command("git", &["worktree", "add", "-b", &branch, &worktree_path])
            .current_dir(&repo_path)
            .output()
    } else {
        shell_command("git", &["worktree", "add", &worktree_path, &branch])
            .current_dir(&repo_path)
            .output()
    };

    let output = output.map_err(|e| format!("Failed to create worktree: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(stderr);
    }

    // Return the new worktree info by listing and finding the new one
    let worktrees = list_worktrees(repo_path).await?;
    let normalized_path = std::path::PathBuf::from(&worktree_path);
    let canonical = normalized_path.canonicalize().ok();

    worktrees
        .into_iter()
        .find(|wt| {
            let wt_canon = std::path::PathBuf::from(&wt.path).canonicalize().ok();
            match (&canonical, &wt_canon) {
                (Some(a), Some(b)) => a == b,
                _ => wt.path == worktree_path,
            }
        })
        .ok_or_else(|| "Worktree created but not found in list".to_string())
}

#[command]
pub async fn remove_worktree(
    repo_path: String,
    worktree_path: String,
    force: bool,
) -> Result<(), String> {
    let args = if force {
        vec!["worktree", "remove", "--force", &worktree_path]
    } else {
        vec!["worktree", "remove", &worktree_path]
    };

    let output = shell_command("git", &args)
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("Failed to remove worktree: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(())
}

// Session history commands

#[command]
pub async fn get_session_history(
    state: State<'_, AppState>,
) -> Result<Vec<SessionHistoryEntry>, String> {
    let db = state.db.lock().await;
    db.get_session_history()
}

#[command]
pub async fn read_log_file(path: String) -> Result<String, String> {
    // Validate path is under the logs directory
    let data_dir = directories::ProjectDirs::from("com", "claudeterminal", "ClaudeTerminal")
        .ok_or("Failed to get project directories")?
        .data_dir()
        .to_path_buf();
    let logs_dir = data_dir.join("logs");
    let canonical_path = std::path::Path::new(&path)
        .canonicalize()
        .map_err(|e| format!("Invalid path: {}", e))?;
    std::fs::create_dir_all(&logs_dir).map_err(|e| format!("Failed to create logs directory: {}", e))?;
    let canonical_logs = logs_dir
        .canonicalize()
        .map_err(|e| format!("Failed to resolve logs directory: {}", e))?;
    if !canonical_path.starts_with(&canonical_logs) {
        return Err("Access denied: path is not under logs directory".to_string());
    }
    std::fs::read_to_string(&canonical_path).map_err(|e| format!("Failed to read log file: {}", e))
}

#[command]
pub async fn delete_session_history(
    state: State<'_, AppState>,
    id: i64,
    log_path: Option<String>,
) -> Result<(), String> {
    // Delete log file if it exists, but only if it's under the logs directory
    if let Some(ref path) = log_path {
        let data_dir = directories::ProjectDirs::from("com", "claudeterminal", "ClaudeTerminal")
            .ok_or("Failed to get project directories")?
            .data_dir()
            .to_path_buf();
        let logs_dir = data_dir.join("logs");
        let _ = std::fs::create_dir_all(&logs_dir);
        if let Ok(canonical_path) = std::path::Path::new(path).canonicalize() {
            if let Ok(canonical_logs) = logs_dir.canonicalize() {
                if canonical_path.starts_with(&canonical_logs) {
                    let _ = std::fs::remove_file(&canonical_path);
                }
            }
        }
    }
    let db = state.db.lock().await;
    db.delete_session_history_entry(id)
}

/// Retrieve the log content for a terminal from a previous session.
/// Looks up the most recent session_history entry for the given terminal_id,
/// reads the log file, and returns its content (capped at 512 KB).
#[command]
pub async fn get_session_log(
    state: State<'_, AppState>,
    terminal_id: String,
) -> Result<Option<String>, String> {
    let log_path = {
        let db = state.db.lock().await;
        db.get_log_path_for_terminal(&terminal_id)?
    };

    let path = match log_path {
        Some(p) => p,
        None => return Ok(None),
    };

    // Validate path is under the logs directory
    let data_dir = directories::ProjectDirs::from("com", "claudeterminal", "ClaudeTerminal")
        .ok_or("Failed to get project directories")?
        .data_dir()
        .to_path_buf();
    let logs_dir = data_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)
        .map_err(|e| format!("Failed to create logs directory: {}", e))?;

    let canonical_path = match std::path::Path::new(&path).canonicalize() {
        Ok(p) => p,
        Err(_) => return Ok(None), // Log file may have been deleted
    };
    let canonical_logs = logs_dir
        .canonicalize()
        .map_err(|e| format!("Failed to resolve logs directory: {}", e))?;
    if !canonical_path.starts_with(&canonical_logs) {
        return Ok(None);
    }

    // Read up to 512 KB
    match std::fs::read(&canonical_path) {
        Ok(bytes) => {
            let max_bytes = 512 * 1024;
            let truncated = if bytes.len() > max_bytes {
                &bytes[bytes.len() - max_bytes..]
            } else {
                &bytes
            };
            Ok(Some(String::from_utf8_lossy(truncated).into_owned()))
        }
        Err(_) => Ok(None),
    }
}

// Snippet commands

#[command]
pub async fn save_snippet(
    state: State<'_, AppState>,
    snippet: Snippet,
) -> Result<(), String> {
    let db = state.db.lock().await;
    db.save_snippet(&snippet)
}

#[command]
pub async fn get_snippets(state: State<'_, AppState>) -> Result<Vec<Snippet>, String> {
    let db = state.db.lock().await;
    db.get_snippets()
}

#[command]
pub async fn delete_snippet(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().await;
    db.delete_snippet(&id)
}

// Claude Global Configuration (~/.claude/)

/// Returns the path to the user's ~/.claude directory
fn get_claude_dir() -> Result<std::path::PathBuf, String> {
    let home = if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE").map_err(|_| "USERPROFILE not set".to_string())?
    } else {
        std::env::var("HOME").map_err(|_| "HOME not set".to_string())?
    };
    Ok(std::path::Path::new(&home).join(".claude"))
}

/// Validates that a filename is safe (no path traversal)
fn validate_filename(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Filename cannot be empty".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") || name.contains('\0') {
        return Err("Invalid filename".to_string());
    }
    Ok(())
}

#[command]
pub async fn read_claude_settings() -> Result<String, String> {
    let settings_path = get_claude_dir()?.join("settings.json");
    if !settings_path.exists() {
        return Ok("{}".to_string());
    }
    std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings.json: {}", e))
}

#[command]
pub async fn write_claude_settings(content: String) -> Result<(), String> {
    // Validate it's valid JSON
    serde_json::from_str::<serde_json::Value>(&content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    let claude_dir = get_claude_dir()?;
    std::fs::create_dir_all(&claude_dir).map_err(|e| e.to_string())?;
    std::fs::write(claude_dir.join("settings.json"), &content)
        .map_err(|e| format!("Failed to write settings.json: {}", e))
}

#[command]
pub async fn list_claude_agents() -> Result<Vec<String>, String> {
    let agents_dir = get_claude_dir()?.join("agents");
    if !agents_dir.exists() {
        return Ok(vec![]);
    }
    let entries = std::fs::read_dir(&agents_dir).map_err(|e| e.to_string())?;
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();
    names.sort();
    Ok(names)
}

#[command]
pub async fn read_claude_agent(name: String) -> Result<String, String> {
    validate_filename(&name)?;
    let path = get_claude_dir()?.join("agents").join(&name);
    if !path.exists() {
        return Err(format!("Agent file not found: {}", name));
    }
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[command]
pub async fn write_claude_agent(name: String, content: String) -> Result<(), String> {
    validate_filename(&name)?;
    let agents_dir = get_claude_dir()?.join("agents");
    std::fs::create_dir_all(&agents_dir).map_err(|e| e.to_string())?;
    std::fs::write(agents_dir.join(&name), &content).map_err(|e| e.to_string())
}

#[command]
pub async fn delete_claude_agent(name: String) -> Result<(), String> {
    validate_filename(&name)?;
    let path = get_claude_dir()?.join("agents").join(&name);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[command]
pub async fn list_claude_commands() -> Result<Vec<String>, String> {
    let commands_dir = get_claude_dir()?.join("commands");
    if !commands_dir.exists() {
        return Ok(vec![]);
    }
    let entries = std::fs::read_dir(&commands_dir).map_err(|e| e.to_string())?;
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();
    names.sort();
    Ok(names)
}

#[command]
pub async fn read_claude_command(name: String) -> Result<String, String> {
    validate_filename(&name)?;
    let path = get_claude_dir()?.join("commands").join(&name);
    if !path.exists() {
        return Err(format!("Command file not found: {}", name));
    }
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[command]
pub async fn write_claude_command(name: String, content: String) -> Result<(), String> {
    validate_filename(&name)?;
    let commands_dir = get_claude_dir()?.join("commands");
    std::fs::create_dir_all(&commands_dir).map_err(|e| e.to_string())?;
    std::fs::write(commands_dir.join(&name), &content).map_err(|e| e.to_string())
}

#[command]
pub async fn delete_claude_command(name: String) -> Result<(), String> {
    validate_filename(&name)?;
    let path = get_claude_dir()?.join("commands").join(&name);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// Telemetry commands

#[command]
pub async fn get_installation_id(state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.lock().await;
    db.get_or_create_installation_id()
}

#[command]
pub async fn send_telemetry_heartbeat(
    state: State<'_, AppState>,
    enabled: bool,
    app_version: String,
) -> Result<(), String> {
    if !enabled {
        return Ok(());
    }
    let installation_id = {
        let db = state.db.lock().await;
        db.get_or_create_installation_id()?
    };
    tokio::spawn(crate::telemetry::send_heartbeat(installation_id, app_version));
    Ok(())
}

// Session summary commands

#[command]
pub async fn summarize_session(log_path: String) -> Result<Option<String>, String> {
    // Validate path is under the logs directory
    let data_dir = directories::ProjectDirs::from("com", "claudeterminal", "ClaudeTerminal")
        .ok_or("Failed to get project directories")?
        .data_dir()
        .to_path_buf();
    let logs_dir = data_dir.join("logs");
    std::fs::create_dir_all(&logs_dir).map_err(|e| format!("Failed to create logs directory: {}", e))?;

    let canonical_path = match std::path::Path::new(&log_path).canonicalize() {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };
    let canonical_logs = logs_dir
        .canonicalize()
        .map_err(|e| format!("Failed to resolve logs directory: {}", e))?;
    if !canonical_path.starts_with(&canonical_logs) {
        return Err("Access denied: path is not under logs directory".to_string());
    }

    // Read log file content (capped at 100KB)
    let bytes = match std::fs::read(&canonical_path) {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };
    let max_bytes = 100 * 1024;
    let truncated = if bytes.len() > max_bytes {
        &bytes[bytes.len() - max_bytes..]
    } else {
        &bytes
    };
    let log_content = String::from_utf8_lossy(truncated);

    // Strip ANSI escape sequences
    let ansi_re = regex::Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]|\x1b\].*?\x07|\x1b\[.*?[A-Za-z]")
        .unwrap();
    let clean_content = ansi_re.replace_all(&log_content, "").to_string();

    if clean_content.trim().is_empty() {
        return Ok(None);
    }

    // Run claude -p to summarize
    let mut cmd = shell_command("claude", &["-p", "--model", "haiku", "Summarize what was accomplished in this terminal session in 2-3 bullet points. Be concise."]);
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return Ok(None), // Claude Code not available
    };

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(clean_content.as_bytes());
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(_) => return Ok(None),
    };

    if !output.status.success() {
        return Ok(None);
    }

    let summary = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if summary.is_empty() {
        return Ok(None);
    }

    Ok(Some(summary))
}

#[command]
pub async fn save_session_summary(
    state: State<'_, AppState>,
    terminal_id: String,
    summary: String,
) -> Result<(), String> {
    let db = state.db.lock().await;
    db.save_session_summary(&terminal_id, &summary)
}

#[command]
pub async fn get_session_summary(
    state: State<'_, AppState>,
    terminal_id: String,
) -> Result<Option<String>, String> {
    let db = state.db.lock().await;
    db.get_session_summary(&terminal_id)
}

// Team tasks command

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskInfo {
    pub id: String,
    pub subject: String,
    pub status: String,
    pub owner: Option<String>,
    pub blocked_by: Vec<String>,
    pub active_form: Option<String>,
}

#[command]
pub async fn get_team_tasks(team_name: String) -> Result<Vec<TaskInfo>, String> {
    // Validate team_name doesn't contain path traversal
    if team_name.contains('/') || team_name.contains('\\') || team_name.contains("..") || team_name.contains('\0') {
        return Err("Invalid team name".to_string());
    }

    let home = if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE").map_err(|_| "USERPROFILE not set".to_string())?
    } else {
        std::env::var("HOME").map_err(|_| "HOME not set".to_string())?
    };

    let tasks_dir = std::path::Path::new(&home)
        .join(".claude")
        .join("tasks")
        .join(&team_name);

    if !tasks_dir.exists() {
        return Ok(vec![]);
    }

    let entries = std::fs::read_dir(&tasks_dir).map_err(|e| e.to_string())?;
    let mut tasks = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Skip .highwatermark and non-JSON files
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || !name.ends_with(".json") {
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let val: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let id = val.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let subject = val.get("subject").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let status = val.get("status").and_then(|v| v.as_str()).unwrap_or("pending").to_string();
        let owner = val.get("owner").and_then(|v| v.as_str()).map(String::from);
        let active_form = val.get("activeForm").and_then(|v| v.as_str()).map(String::from);
        let blocked_by: Vec<String> = val.get("blockedBy")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        if !id.is_empty() {
            tasks.push(TaskInfo {
                id,
                subject,
                status,
                owner,
                blocked_by,
                active_form,
            });
        }
    }

    // Sort by id
    tasks.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(tasks)
}

// Memory & CLAUDE.md commands

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFileInfo {
    pub path: String,
    pub name: String,
    pub project: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeMdInfo {
    pub path: String,
    pub scope: String,
    pub project_name: Option<String>,
}

/// Validates that a path is under ~/.claude/
fn validate_claude_path(path: &str) -> Result<(), String> {
    let claude_dir = get_claude_dir()?;
    let canonical_claude = claude_dir
        .canonicalize()
        .unwrap_or_else(|_| claude_dir.clone());
    let target = std::path::Path::new(path);
    let canonical_target = target
        .canonicalize()
        .unwrap_or_else(|_| target.to_path_buf());
    if !canonical_target.starts_with(&canonical_claude) {
        return Err("Access denied: path is not under ~/.claude/".to_string());
    }
    Ok(())
}

#[command]
pub async fn list_memory_files(project_path: Option<String>) -> Result<Vec<MemoryFileInfo>, String> {
    let claude_dir = get_claude_dir()?;
    let projects_dir = claude_dir.join("projects");

    if !projects_dir.exists() {
        return Ok(vec![]);
    }

    let mut files = Vec::new();

    let scan_project = |project_dir: &std::path::Path, files: &mut Vec<MemoryFileInfo>| {
        let memory_dir = project_dir.join("memory");
        if !memory_dir.exists() || !memory_dir.is_dir() {
            return;
        }
        let project_name = project_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if let Ok(entries) = std::fs::read_dir(&memory_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    files.push(MemoryFileInfo {
                        path: path.to_string_lossy().to_string(),
                        name,
                        project: project_name.clone(),
                        size,
                    });
                }
            }
        }
    };

    if let Some(ref specific_project) = project_path {
        // Scan only the specific project
        let target = std::path::Path::new(specific_project);
        if target.exists() && target.is_dir() {
            scan_project(target, &mut files);
        }
    } else {
        // Scan all projects
        if let Ok(entries) = std::fs::read_dir(&projects_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    scan_project(&path, &mut files);
                }
            }
        }
    }

    Ok(files)
}

#[command]
pub async fn read_memory_file(path: String) -> Result<String, String> {
    validate_claude_path(&path)?;
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read memory file: {}", e))
}

#[command]
pub async fn write_memory_file(path: String, content: String) -> Result<(), String> {
    validate_claude_path(&path)?;
    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, &content).map_err(|e| format!("Failed to write memory file: {}", e))
}

#[command]
pub async fn list_claude_md_files() -> Result<Vec<ClaudeMdInfo>, String> {
    let mut files = Vec::new();
    let claude_dir = get_claude_dir()?;

    // Global ~/.claude/CLAUDE.md
    let global_md = claude_dir.join("CLAUDE.md");
    if global_md.exists() {
        files.push(ClaudeMdInfo {
            path: global_md.to_string_lossy().to_string(),
            scope: "global".to_string(),
            project_name: None,
        });
    }

    // Project-level CLAUDE.md files in ~/.claude/projects/*/
    let projects_dir = claude_dir.join("projects");
    if projects_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&projects_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let md_path = path.join("CLAUDE.md");
                    if md_path.exists() {
                        let project_name = entry.file_name().to_string_lossy().to_string();
                        files.push(ClaudeMdInfo {
                            path: md_path.to_string_lossy().to_string(),
                            scope: "project".to_string(),
                            project_name: Some(project_name),
                        });
                    }
                }
            }
        }
    }

    Ok(files)
}

// Agent Teams (multi-agent orchestration)

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TeamMember {
    pub agent_id: String,
    pub name: String,
    pub agent_type: String,
    pub model: Option<String>,
    pub joined_at: Option<u64>,
    pub cwd: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TeamConfig {
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<u64>,
    pub lead_agent_id: Option<String>,
    pub members: Vec<TeamMember>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TeamInfo {
    pub dir_name: String,
    pub config: TeamConfig,
    pub task_count: Option<u32>,
}

#[command]
pub async fn get_active_teams() -> Result<Vec<TeamInfo>, String> {
    let home = if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE").map_err(|_| "USERPROFILE not set".to_string())?
    } else {
        std::env::var("HOME").map_err(|_| "HOME not set".to_string())?
    };

    let teams_dir = std::path::Path::new(&home).join(".claude").join("teams");
    if !teams_dir.exists() {
        return Ok(vec![]);
    }

    let entries = std::fs::read_dir(&teams_dir).map_err(|e| e.to_string())?;
    let mut teams = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let config_path = path.join("config.json");
        if !config_path.exists() {
            continue;
        }

        let config_str = match std::fs::read_to_string(&config_path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let config: TeamConfig = match serde_json::from_str(&config_str) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let dir_name = entry.file_name().to_string_lossy().to_string();

        // Read task count from .highwatermark
        let tasks_dir = std::path::Path::new(&home)
            .join(".claude")
            .join("tasks")
            .join(&dir_name);
        let hwm_path = tasks_dir.join(".highwatermark");
        let task_count = std::fs::read_to_string(&hwm_path)
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok());

        teams.push(TeamInfo {
            dir_name,
            config,
            task_count,
        });
    }

    Ok(teams)
}

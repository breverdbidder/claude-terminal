use crate::config::{ConfigProfile, HintCategory};
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

    let config = {
        let mut terminals = state.terminals.lock().await;
        terminals.create_terminal(
            request.label,
            request.working_directory,
            request.claude_args,
            request.env_vars,
            request.color_tag,
            request.nickname,
            tx,
        )?
    };

    let terminal_id = config.id.clone();

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

        // Terminal process exited — notify the frontend
        if let Err(e) = app_clone.emit("terminal-finished", serde_json::json!({
            "id": terminal_id,
        })) {
            eprintln!("Failed to emit terminal-finished: {}", e);
        }
    });

    Ok(config)
}

#[command]
pub async fn write_to_terminal(
    state: State<'_, AppState>,
    id: String,
    data: Vec<u8>,
) -> Result<(), String> {
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
        let mut cmd = std::process::Command::new(program);
        for arg in args {
            cmd.arg(arg);
        }
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
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn open_external_url(url: String) -> Result<(), String> {
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

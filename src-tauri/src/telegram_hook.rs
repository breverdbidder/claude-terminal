//! Telegram exit-code hook for ClaudeTerminal sessions.
//!
//! Watches each spawned child process and fires a Telegram notification when
//! the process exits with a non-zero status (excluding common user-interrupt
//! codes 0/130/143). Non-fatal: silently no-ops if the bot token is not
//! configured, and never blocks terminal creation.
//!
//! Runtime configuration via env vars (read at notification time, not startup):
//!   TELEGRAM_BOT_TOKEN  -- bot API token (REQUIRED for notifications to fire)
//!   TELEGRAM_CHAT_ID    -- chat ID to send to (defaults to 740118343)
//!
//! See BASELINE.md -> "Telegram exit-code notifications" for setup steps.

use serde_json::json;

const DEFAULT_CHAT_ID: &str = "740118343"; // BidDeedAI_bot DM with Ariel

/// Spawn a background task that calls `child.wait()` and, on non-zero exit
/// (excluding 0/130/143), sends a Telegram notification. Drops the watcher
/// silently if no tokio runtime is available or no bot token is configured.
pub fn watch_child_exit(
    child: Box<dyn portable_pty::Child + Send + Sync>,
    terminal_id: String,
    description: String,
) {
    let handle = match tokio::runtime::Handle::try_current() {
        Ok(h) => h,
        Err(_) => {
            eprintln!(
                "[telegram_hook] no tokio runtime; exit watcher skipped for {}",
                terminal_id
            );
            return;
        }
    };

    handle.spawn(async move {
        // child.wait() is blocking; run on the blocking pool.
        let join_result = tokio::task::spawn_blocking(move || {
            let mut child = child;
            child.wait()
        })
        .await;

        let exit_status = match join_result {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                eprintln!("[telegram_hook] wait failed for {}: {}", terminal_id, e);
                return;
            }
            Err(e) => {
                eprintln!("[telegram_hook] join failed for {}: {}", terminal_id, e);
                return;
            }
        };

        let exit_code = exit_status.exit_code();

        // Filter common user-driven exits to avoid notification spam:
        //   0   = clean exit
        //   130 = SIGINT (user pressed Ctrl+C)
        //   143 = SIGTERM (user closed the terminal cell)
        if matches!(exit_code, 0 | 130 | 143) {
            return;
        }

        send_telegram(&terminal_id, &description, exit_code).await;
    });
}

async fn send_telegram(terminal_id: &str, description: &str, exit_code: u32) {
    let bot_token = match std::env::var("TELEGRAM_BOT_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => return, // No-op if not configured
    };
    let chat_id =
        std::env::var("TELEGRAM_CHAT_ID").unwrap_or_else(|_| DEFAULT_CHAT_ID.to_string());

    let host = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown-host".to_string());

    let id_short: String = terminal_id.chars().take(8).collect();

    let text = format!(
        "\u{1F534} *ClaudeTerminal session exited*\n\n*Description:* `{}`\n*Terminal ID:* `{}`\n*Exit code:* `{}`\n*Host:* `{}`",
        description, id_short, exit_code, host
    );

    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    let body = json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "Markdown",
    });

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[telegram_hook] failed to build reqwest client: {}", e);
            return;
        }
    };

    match client.post(&url).json(&body).send().await {
        Ok(resp) if resp.status().is_success() => {}
        Ok(resp) => {
            eprintln!(
                "[telegram_hook] Telegram API non-success: HTTP {}",
                resp.status()
            );
        }
        Err(e) => {
            eprintln!("[telegram_hook] Telegram POST failed: {}", e);
        }
    }
}

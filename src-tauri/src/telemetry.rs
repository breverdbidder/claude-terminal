use serde::Serialize;

const WORKER_URL: &str = "https://ct-analytics.claude-terminal.workers.dev";

// Token baked in at build time. If unset/empty, heartbeats are skipped so
// unsigned/dev builds never post unauthenticated traffic.
const INGEST_TOKEN: Option<&str> = option_env!("CT_INGEST_TOKEN");

#[derive(Serialize)]
struct HeartbeatPayload {
    installation_id: String,
    app_version: String,
    os: String,
    os_version: String,
    timestamp: String,
}

pub async fn send_heartbeat(installation_id: String, app_version: String) {
    let token = match INGEST_TOKEN {
        Some(t) if !t.is_empty() => t,
        _ => {
            eprintln!("[telemetry] CT_INGEST_TOKEN not set at build time; skipping heartbeat");
            return;
        }
    };

    let os = std::env::consts::OS.to_string();
    let os_version = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);

    let payload = HeartbeatPayload {
        installation_id,
        app_version,
        os,
        os_version,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[telemetry] Failed to create HTTP client: {}", e);
            return;
        }
    };

    match client
        .post(format!("{}/heartbeat", WORKER_URL))
        .header("x-ct-token", token)
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            eprintln!("[telemetry] Heartbeat sent (status: {})", resp.status());
        }
        Err(e) => {
            eprintln!("[telemetry] Heartbeat failed: {}", e);
        }
    }
}

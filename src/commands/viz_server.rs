use crate::state::delta::{GraphDelta, GraphSnapshot};
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use futures_util::{SinkExt, StreamExt};
use miette::{IntoDiagnostic, Result, miette};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::time::interval;
use tracing::info;

const ARC_DIAGRAM_HTML: &str = include_str!("../../templates/arc_diagram.html");

pub fn execute_viz_server(port: u16, bind: String, open: bool, stop: bool) -> Result<()> {
    // Resolve repository root dynamically to ensure PID is found correctly
    // regardless of where in the repo the command is executed.
    let current_dir = std::env::current_dir().into_diagnostic()?;
    let layout = if let Ok(repo) = gix::discover(&current_dir) {
        Layout::new(
            repo.workdir()
                .unwrap_or_else(|| repo.path())
                .to_string_lossy()
                .as_ref(),
        )
    } else {
        Layout::new(current_dir.to_string_lossy().as_ref())
    };

    if stop {
        return kill_viz_server(&layout);
    }

    validate_bind(&bind)?;

    // Write PID file so --stop can find us later
    write_pid_file(&layout)?;

    let url = format!("http://{}:{}", bind, port);
    println!("Starting viz server at {}", url);

    if open && let Err(e) = webbrowser::open(&url) {
        tracing::warn!("Failed to open browser: {}", e);
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .into_diagnostic()?;

    let result = rt.block_on(start_server(bind, port));

    // Always clean up PID file on exit
    remove_pid_file(&layout);
    result
}

fn write_pid_file(layout: &Layout) -> Result<()> {
    let pid = std::process::id();
    let path = layout.pid_file();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, pid.to_string())
        .map_err(|e| miette!("Failed to write PID file {}: {}", path, e))?;
    Ok(())
}

fn read_pid_file(layout: &Layout) -> Result<Option<u32>> {
    let path = layout.pid_file();
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| miette!("Failed to read PID file {}: {}", path, e))?;
    let pid: u32 = content
        .trim()
        .parse()
        .map_err(|e| miette!("Invalid PID in {}: {}", path, e))?;
    Ok(Some(pid))
}

fn remove_pid_file(layout: &Layout) {
    let path = layout.pid_file();
    let _ = std::fs::remove_file(&path);
}

fn kill_viz_server(layout: &Layout) -> Result<()> {
    match read_pid_file(layout)? {
        Some(pid) => {
            println!("Stopping viz server (PID {})...", pid);
            #[cfg(target_os = "windows")]
            {
                // CR6: Validate the exact image name from the tasklist CSV output.
                // tasklist /FO CSV /NH emits lines like:
                //   "changeguard.exe","1234","Console","1","12,345 K"
                // We require the image name field to match our binary exactly to
                // prevent false positives from processes that have "changeguard" as
                // a substring of their argv or working directory.
                let expected_image = std::env::current_exe()
                    .ok()
                    .and_then(|p| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.to_lowercase())
                    })
                    .unwrap_or_else(|| "changeguard.exe".to_string());

                let verify_output = std::process::Command::new("tasklist")
                    .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
                    .output()
                    .map_err(|e| miette!("Failed to verify process: {}", e))?;
                let out_str = String::from_utf8_lossy(&verify_output.stdout);

                let is_our_process = out_str.lines().any(|line| {
                    // CSV first field is the image name (quoted).
                    let image = line
                        .split(',')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"');
                    image.to_lowercase() == expected_image
                });

                if !is_our_process {
                    println!(
                        "Process {} is not {} (may have exited).",
                        pid, expected_image
                    );
                    remove_pid_file(layout);
                    return Ok(());
                }

                let output = std::process::Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/F"])
                    .output()
                    .map_err(|e| miette!("Failed to run taskkill: {}", e))?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("not found") || stderr.contains("does not exist") {
                        println!("Process {} not found (already exited).", pid);
                    } else {
                        return Err(miette!(
                            "taskkill failed ({}): {}",
                            output.status,
                            stderr.trim()
                        ));
                    }
                } else {
                    println!("Viz server stopped.");
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                // Verify process name first via ps
                let verify_output = std::process::Command::new("ps")
                    .args(["-p", &pid.to_string(), "-o", "comm="])
                    .output()
                    .map_err(|e| miette!("Failed to verify process: {}", e))?;

                let out_str = String::from_utf8_lossy(&verify_output.stdout);
                if !out_str.to_lowercase().contains("changeguard") {
                    println!("Process {} is not changeguard (may have exited).", pid);
                    remove_pid_file(layout);
                    return Ok(());
                }

                let output = std::process::Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .output()
                    .map_err(|e| miette!("Failed to run kill: {}", e))?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("No such process") {
                        println!("Process {} not found (already exited).", pid);
                    } else {
                        return Err(miette!(
                            "kill failed ({}): {}",
                            output.status,
                            stderr.trim()
                        ));
                    }
                } else {
                    println!("Viz server stopped.");
                }
            }
            remove_pid_file(layout);
            Ok(())
        }
        None => {
            println!("No viz server PID file found. Server may not be running.");
            Ok(())
        }
    }
}

fn validate_bind(bind: &str) -> Result<()> {
    if bind == "127.0.0.1" || bind == "::1" || bind == "localhost" {
        return Ok(());
    }
    if let Ok(addr) = bind.parse::<std::net::IpAddr>()
        && addr.is_loopback()
    {
        return Ok(());
    }
    Err(miette!(
        "Server must bind to a loopback address (127.0.0.1 or ::1). Got: {}",
        bind
    ))
}

async fn start_server(bind: String, port: u16) -> Result<()> {
    let addr = SocketAddr::new(
        bind.parse()
            .map_err(|e| miette!("Invalid bind address: {}", e))?,
        port,
    );
    let listener = TcpListener::bind(addr).await.into_diagnostic()?;
    info!("Viz server listening on {}", addr);

    let snapshot = Arc::new(RwLock::new(None::<GraphSnapshot>));
    let (broadcast_tx, _broadcast_rx) = broadcast::channel::<String>(16);

    let current_dir = std::env::current_dir().into_diagnostic()?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let db_path = layout.state_subdir().join("ledger.db");

    // Spawn polling task
    let snapshot_clone = Arc::clone(&snapshot);
    let broadcast_tx_clone = broadcast_tx.clone();
    let db_path_clone = db_path.clone().into_std_path_buf();
    tokio::spawn(async move {
        run_polling(snapshot_clone, broadcast_tx_clone, db_path_clone).await;
    });

    // Spawn heartbeat task
    let broadcast_tx_hb = broadcast_tx.clone();
    tokio::spawn(async move {
        let mut heartbeat = interval(Duration::from_secs(30));
        loop {
            heartbeat.tick().await;
            let msg = serde_json::json!({
                "type": "heartbeat",
                "timestamp": format!("{}", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")),
            })
            .to_string();
            let _ = broadcast_tx_hb.send(msg);
        }
    });

    let html = ARC_DIAGRAM_HTML.to_string();

    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            _ = &mut shutdown => {
                info!("Shutdown signal received");
                break;
            }
            result = listener.accept() => {
                let (stream, peer_addr) = match result {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::warn!("Accept error: {}", e);
                        continue;
                    }
                };

                let snapshot = Arc::clone(&snapshot);
                let broadcast_tx = broadcast_tx.clone();
                let html = html.clone();

                tokio::spawn(async move {
                    handle_connection(stream, peer_addr, snapshot, broadcast_tx, html).await;
                });
            }
        }
    }

    Ok(())
}

async fn run_polling(
    snapshot: Arc<RwLock<Option<GraphSnapshot>>>,
    broadcast_tx: broadcast::Sender<String>,
    db_path: std::path::PathBuf,
) {
    let storage = match StorageManager::init(&db_path) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to initialize storage: {}", e);
            return;
        }
    };

    let cozo = match storage.cozo.as_ref() {
        Some(c) => c,
        None => {
            tracing::error!("CozoDB not initialized. Run 'index' first.");
            return;
        }
    };

    let mut last_snapshot: Option<GraphSnapshot> = None;
    let mut tick = interval(Duration::from_millis(250));

    loop {
        tick.tick().await;

        if broadcast_tx.receiver_count() == 0 {
            last_snapshot = None;
            continue;
        }

        let current = match GraphSnapshot::from_cozo(cozo) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to query graph: {}", e);
                continue;
            }
        };

        if let Some(ref last) = last_snapshot {
            let delta = last.diff(&current);
            if !delta.is_empty() {
                let msg = build_delta_message(&delta);
                let _ = broadcast_tx.send(msg);
            }
        }

        {
            let mut guard = snapshot.write().await;
            *guard = Some(current.clone());
        }
        last_snapshot = Some(current);
    }
}

fn build_delta_message(delta: &GraphDelta) -> String {
    serde_json::json!({
        "type": "delta",
        "timestamp": format!("{}", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")),
        "nodes": {
            "added": delta.added_nodes,
            "removed": delta.removed_nodes,
            "updated": delta.updated_nodes,
        },
        "edges": {
            "added": delta.added_edges,
            "removed": delta.removed_edges,
        }
    })
    .to_string()
}

fn build_snapshot_message(snap: &GraphSnapshot) -> String {
    let mut nodes: Vec<_> = snap.nodes.values().cloned().collect();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    let mut edges: Vec<_> = snap.edges.iter().cloned().collect();
    edges.sort_by(|a, b| (&a.from, &a.to, &a.label).cmp(&(&b.from, &b.to, &b.label)));

    serde_json::json!({
        "type": "snapshot",
        "timestamp": format!("{}", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")),
        "nodes": nodes,
        "edges": edges,
    })
    .to_string()
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    peer_addr: std::net::SocketAddr,
    snapshot: Arc<RwLock<Option<GraphSnapshot>>>,
    broadcast_tx: broadcast::Sender<String>,
    html: String,
) {
    let mut peek_buf = [0u8; 4096];
    let n = match stream.peek(&mut peek_buf).await {
        Ok(n) => n,
        Err(e) => {
            tracing::warn!("Peek failed for {}: {}", peer_addr, e);
            return;
        }
    };
    let req_str = String::from_utf8_lossy(&peek_buf[..n]);

    if req_str.contains("Upgrade: websocket") && req_str.contains("GET /ws") {
        match tokio_tungstenite::accept_async(stream).await {
            Ok(ws) => {
                info!("WebSocket client connected: {}", peer_addr);
                handle_ws_client(ws, snapshot, broadcast_tx).await;
                info!("WebSocket client disconnected: {}", peer_addr);
            }
            Err(e) => {
                tracing::warn!("WebSocket accept failed for {}: {}", peer_addr, e);
            }
        }
    } else if req_str.starts_with("GET / ") || req_str.starts_with("GET / HTTP") {
        if let Err(e) = serve_http(stream, &html).await {
            tracing::warn!("HTTP serve failed for {}: {}", peer_addr, e);
        }
    } else {
        // Ignore other requests
    }
}

async fn serve_http(mut stream: tokio::net::TcpStream, body: &str) -> std::io::Result<()> {
    use tokio::io::AsyncWriteExt;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

async fn handle_ws_client(
    mut ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    snapshot: Arc<RwLock<Option<GraphSnapshot>>>,
    broadcast_tx: broadcast::Sender<String>,
) {
    let snap = {
        let guard = snapshot.read().await;
        guard.clone()
    };

    if let Some(snap) = snap {
        let msg = build_snapshot_message(&snap);
        if ws
            .send(tokio_tungstenite::tungstenite::Message::Text(msg.into()))
            .await
            .is_err()
        {
            return;
        }
    }

    let mut rx = broadcast_tx.subscribe();
    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(text) => {
                        if ws.send(tokio_tungstenite::tungstenite::Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            ws_msg = ws.next() => {
                match ws_msg {
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) => break,
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Ping(data))) => {
                        let _ = ws.send(tokio_tungstenite::tungstenite::Message::Pong(data)).await;
                    }
                    Some(Err(_)) => break,
                    None => break,
                    _ => {}
                }
            }
        }
    }

    let _ = ws.close(None).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loopback_allowed() {
        assert!(validate_bind("127.0.0.1").is_ok());
        assert!(validate_bind("::1").is_ok());
        assert!(validate_bind("localhost").is_ok());
    }

    #[test]
    fn test_non_loopback_rejected() {
        assert!(validate_bind("0.0.0.0").is_err());
        assert!(validate_bind("192.168.1.1").is_err());
        assert!(validate_bind("::").is_err());
        assert!(validate_bind("10.0.0.1").is_err());
    }

    #[test]
    fn test_pid_file_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let root = camino::Utf8Path::from_path(tmp.path()).unwrap();
        let layout = Layout::new(root);

        // Clean slate
        remove_pid_file(&layout);
        assert!(read_pid_file(&layout).unwrap().is_none());

        // Write and read back
        write_pid_file(&layout).unwrap();
        let pid = read_pid_file(&layout).unwrap();
        assert_eq!(pid, Some(std::process::id()));

        // Remove
        remove_pid_file(&layout);
        assert!(read_pid_file(&layout).unwrap().is_none());
    }

    #[test]
    fn test_kill_no_pid_file() {
        let tmp = tempfile::tempdir().unwrap();
        let root = camino::Utf8Path::from_path(tmp.path()).unwrap();
        let layout = Layout::new(root);
        remove_pid_file(&layout);
        // Should succeed gracefully when no server is running
        assert!(kill_viz_server(&layout).is_ok());
    }
}

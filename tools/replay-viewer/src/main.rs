/// Game State Stepper — Developer Replay Viewer
///
/// Axum HTTP server that loads game script JSON files, pre-computes all step
/// snapshots via `ReplaySession::from_script()`, and serves them via a REST API
/// consumed by the Svelte frontend.
///
/// Usage:
///   replay-viewer --scripts-dir test-data/generated-scripts/
///   replay-viewer --script path/to/script.json --port 3030
mod api;
mod replay;
mod view_model;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use mtg_engine::testing::script_schema::GameScript;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

use api::{AppState, SharedState};

/// Game State Stepper — Developer Replay Viewer
#[derive(Parser, Debug)]
#[command(name = "replay-viewer", about = "MTG Commander game state stepper")]
struct Cli {
    /// Path to a game script JSON file to load on startup.
    #[arg(long)]
    script: Option<PathBuf>,

    /// Directory containing game script JSON files.
    #[arg(long, default_value = "test-data/generated-scripts")]
    scripts_dir: PathBuf,

    /// Port to bind the HTTP server to.
    #[arg(long, default_value = "3030")]
    port: u16,

    /// Host address to bind to. Use 0.0.0.0 to expose on the local network.
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Resolve scripts_dir to an absolute path.
    let scripts_dir = cli.scripts_dir.canonicalize().unwrap_or(cli.scripts_dir);

    // Build shared state.
    let shared_state: SharedState = Arc::new(RwLock::new(AppState::new(scripts_dir.clone())));

    // If a script was specified on the command line, load it now.
    if let Some(script_path) = &cli.script {
        println!("Loading script: {}", script_path.display());
        let json = std::fs::read_to_string(script_path)
            .with_context(|| format!("Failed to read {}", script_path.display()))?;
        let script: GameScript = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse {}", script_path.display()))?;

        println!("Script: {} ({})", script.metadata.name, script.metadata.id);

        let session = replay::ReplaySession::from_script(&script)
            .context("Failed to build replay session")?;

        println!(
            "Replay session built: {} steps (including step 0 = initial state)",
            session.step_count()
        );

        let mut state = shared_state.write().await;
        state.session = Some(session);
    } else {
        // Auto-load the first script found.
        if let Some(first_script) = find_first_script(&scripts_dir) {
            println!("Auto-loading first script: {}", first_script.display());
            if let Ok(json) = std::fs::read_to_string(&first_script) {
                if let Ok(script) = serde_json::from_str::<GameScript>(&json) {
                    if let Ok(session) = replay::ReplaySession::from_script(&script) {
                        println!(
                            "Auto-loaded: {} ({} steps)",
                            script.metadata.name,
                            session.step_count()
                        );
                        let mut state = shared_state.write().await;
                        state.session = Some(session);
                    }
                }
            }
        }
    }

    // Build the axum router.
    let dist_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|pp| pp.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("dist");

    // Also try a local dist/ relative to cwd (for `cargo run` from workspace root).
    let dist_dir = if dist_dir.exists() {
        dist_dir
    } else {
        let cwd_dist = PathBuf::from("tools/replay-viewer/dist");
        if cwd_dist.exists() {
            cwd_dist
        } else {
            PathBuf::from("dist")
        }
    };

    let router = build_router(shared_state, &dist_dir);

    let addr = format!("{}:{}", cli.host, cli.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind to {addr}"))?;

    println!("Replay viewer running at http://{}:{}/", cli.host, cli.port);
    println!("API: http://{}:{}/api/", cli.host, cli.port);
    if dist_dir.exists() {
        println!("Frontend: serving from {}", dist_dir.display());
    } else {
        println!(
            "Frontend: dist/ not found — run `npm run build` in tools/replay-viewer/frontend/"
        );
    }

    axum::serve(listener, router).await?;
    Ok(())
}

/// Build the axum router with all API routes and static file serving.
fn build_router(state: SharedState, dist_dir: &PathBuf) -> Router {
    let api_router = Router::new()
        .route("/scripts", get(api::get_scripts))
        .route("/session", get(api::get_session))
        .route("/step/:n", get(api::get_step))
        .route("/step/:n/state", get(api::get_step_state))
        .route("/load", post(api::post_load))
        .with_state(state);

    let router = Router::new().nest("/api", api_router);

    // Serve the Svelte frontend from dist/ if it exists.
    if dist_dir.exists() {
        router.fallback_service(ServeDir::new(dist_dir).append_index_html_on_directories(true))
    } else {
        router
    }
}

// ── Integration tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use mtg_engine::testing::script_schema::GameScript;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tower::ServiceExt;

    const SCRIPT_PATH: &str =
        "../../test-data/generated-scripts/baseline/001_priority_pass_empty_stack.json";

    fn load_script() -> GameScript {
        let json = std::fs::read_to_string(SCRIPT_PATH)
            .unwrap_or_else(|_| panic!("Failed to read {SCRIPT_PATH}"));
        serde_json::from_str(&json).expect("Failed to parse script")
    }

    fn make_empty_state() -> SharedState {
        Arc::new(RwLock::new(AppState::new(PathBuf::from("."))))
    }

    fn make_loaded_state() -> SharedState {
        let script = load_script();
        let session = replay::ReplaySession::from_script(&script).unwrap();
        let scripts_dir =
            PathBuf::from("../../test-data/generated-scripts").canonicalize().unwrap();
        let mut state = AppState::new(scripts_dir);
        state.session = Some(session);
        Arc::new(RwLock::new(state))
    }

    fn app(state: SharedState) -> Router {
        build_router(state, &PathBuf::from("nonexistent_dist"))
    }

    async fn body_json(body: axum::body::Body) -> serde_json::Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    // ── GET /api/session ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_session_no_script_returns_not_loaded() {
        let resp = app(make_empty_state())
            .oneshot(Request::builder().uri("/api/session").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        assert_eq!(json["loaded"], false);
        assert_eq!(json["total_steps"], 0);
    }

    #[tokio::test]
    async fn test_get_session_with_loaded_script() {
        let resp = app(make_loaded_state())
            .oneshot(Request::builder().uri("/api/session").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        assert_eq!(json["loaded"], true);
        assert!(json["total_steps"].as_u64().unwrap() >= 2, "must have at least 2 steps");
    }

    // ── GET /api/step/:n ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_step_zero_returns_state() {
        let resp = app(make_loaded_state())
            .oneshot(Request::builder().uri("/api/step/0").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        assert_eq!(json["index"], 0);
        // Step 0 is the initial state — all players at 40 life.
        // players is a HashMap<String, PlayerView> (object keyed by player name).
        let players = json["state"]["players"].as_object().unwrap();
        assert!(!players.is_empty(), "must have at least one player");
        for (_name, p) in players {
            assert_eq!(p["life"], 40, "initial life should be 40 for all players");
        }
    }

    #[tokio::test]
    async fn test_get_step_out_of_range_returns_404() {
        let resp = app(make_loaded_state())
            .oneshot(Request::builder().uri("/api/step/9999").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_step_no_session_returns_404() {
        let resp = app(make_empty_state())
            .oneshot(Request::builder().uri("/api/step/0").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // ── GET /api/scripts ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_scripts_returns_groups() {
        let resp = app(make_loaded_state())
            .oneshot(Request::builder().uri("/api/scripts").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        let total = json["total"].as_u64().unwrap();
        assert!(total > 0, "should find at least one script");
        assert!(json["groups"].is_object(), "groups must be an object");
    }

    // ── POST /api/load — security ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_post_load_rejects_absolute_path() {
        let body = serde_json::json!({"path": "/etc/passwd"}).to_string();
        let resp = app(make_loaded_state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/load")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_post_load_rejects_path_traversal() {
        let body = serde_json::json!({"path": "../../etc/passwd"}).to_string();
        let resp = app(make_loaded_state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/load")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        // Canonicalize fails (file doesn't exist) → 404, or path check fails → 400.
        let status = resp.status();
        assert!(
            status == StatusCode::NOT_FOUND || status == StatusCode::BAD_REQUEST,
            "path traversal must be rejected with 400 or 404, got {status}"
        );
    }

    #[tokio::test]
    async fn test_post_load_valid_script() {
        let body = serde_json::json!({"path": "baseline/001_priority_pass_empty_stack.json"})
            .to_string();
        let resp = app(make_loaded_state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/load")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp.into_body()).await;
        assert_eq!(json["loaded"], true);
    }
}

/// Find the first JSON script in a directory tree (deterministic: sorted order).
fn find_first_script(dir: &PathBuf) -> Option<PathBuf> {
    let mut entries: Vec<_> = std::fs::read_dir(dir).ok()?.flatten().collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_first_script(&path) {
                return Some(found);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
            return Some(path);
        }
    }
    None
}

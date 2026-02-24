/// Axum route handlers for the replay viewer API.
///
/// All handlers share `Arc<RwLock<AppState>>` via axum's `State` extractor.
/// Heavy work (replay computation) is done at load time in `POST /api/load`
/// so that `GET /api/step/:n` is a pure O(1) data lookup.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::replay::{AssertionResult, ReplaySession};
use crate::view_model::{AssertionResultView, StateViewModel, StepViewModel};

// ── AppState ──────────────────────────────────────────────────────────────────

/// Shared application state behind `Arc<RwLock<AppState>>`.
pub struct AppState {
    /// The currently loaded replay session (None until a script is loaded).
    pub session: Option<ReplaySession>,
    /// Directory to scan for game script JSON files.
    pub scripts_dir: PathBuf,
}

/// Type alias for the shared state handle.
pub type SharedState = Arc<RwLock<AppState>>;

impl AppState {
    pub fn new(scripts_dir: PathBuf) -> Self {
        AppState {
            session: None,
            scripts_dir,
        }
    }
}

// ── Response types ────────────────────────────────────────────────────────────

/// Metadata about the current session returned by `GET /api/session`.
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub loaded: bool,
    pub script_id: Option<String>,
    pub script_name: Option<String>,
    pub description: Option<String>,
    pub total_steps: usize,
    pub players: Vec<String>,
}

/// A script entry in the script listing.
#[derive(Debug, Serialize)]
pub struct ScriptEntry {
    pub path: String,
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub review_status: String,
    pub subdirectory: String,
}

/// Response for `GET /api/scripts`.
#[derive(Debug, Serialize)]
pub struct ScriptsResponse {
    /// Scripts grouped by subdirectory.
    pub groups: HashMap<String, Vec<ScriptEntry>>,
    pub total: usize,
}

/// Request body for `POST /api/load`.
#[derive(Debug, Deserialize)]
pub struct LoadRequest {
    /// Path to the script, relative to `scripts_dir` (or absolute).
    pub path: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// `GET /api/session` — return metadata about the currently loaded session.
pub async fn get_session(State(state): State<SharedState>) -> Json<SessionResponse> {
    let state = state.read().await;
    match &state.session {
        None => Json(SessionResponse {
            loaded: false,
            script_id: None,
            script_name: None,
            description: None,
            total_steps: 0,
            players: vec![],
        }),
        Some(session) => {
            let players: Vec<String> = session.player_map.keys().cloned().collect();
            Json(SessionResponse {
                loaded: true,
                script_id: Some(session.script.metadata.id.clone()),
                script_name: Some(session.script.metadata.name.clone()),
                description: Some(session.script.metadata.description.clone()),
                total_steps: session.step_count(),
                players,
            })
        }
    }
}

/// `GET /api/step/:n` — return the full `StepViewModel` for step N.
pub async fn get_step(
    State(state): State<SharedState>,
    AxumPath(n): AxumPath<usize>,
) -> Result<Json<StepViewModel>, (StatusCode, String)> {
    let state = state.read().await;
    let session = state
        .session
        .as_ref()
        .ok_or((StatusCode::NOT_FOUND, "No session loaded".to_string()))?;

    let snap = session.steps.get(n).ok_or((
        StatusCode::NOT_FOUND,
        format!("Step {n} not found (total: {})", session.step_count()),
    ))?;

    let state_vm = StateViewModel::from_game_state(&snap.state_after, &session.player_names);

    let assertions_view = snap
        .assertions
        .as_ref()
        .map(|results| results.iter().map(assertion_to_view).collect());

    let script_action = serde_json::to_value(&snap.script_action).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to serialize script_action: {e}"),
        )
    })?;

    let command = snap
        .command
        .as_ref()
        .map(|c| {
            serde_json::to_value(c).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to serialize command: {e}"),
                )
            })
        })
        .transpose()?;

    let events: Vec<serde_json::Value> = snap
        .events
        .iter()
        .enumerate()
        .map(|(i, e)| {
            serde_json::to_value(e).map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to serialize event[{i}]: {err}"),
                )
            })
        })
        .collect::<Result<_, _>>()?;

    let vm = StepViewModel {
        index: snap.index,
        total_steps: session.step_count(),
        script_action,
        command,
        events,
        state: state_vm,
        assertions: assertions_view,
    };

    Ok(Json(vm))
}

/// `GET /api/step/:n/state` — return only the `StateViewModel` (lighter payload).
pub async fn get_step_state(
    State(state): State<SharedState>,
    AxumPath(n): AxumPath<usize>,
) -> Result<Json<StateViewModel>, (StatusCode, String)> {
    let state = state.read().await;
    let session = state
        .session
        .as_ref()
        .ok_or((StatusCode::NOT_FOUND, "No session loaded".to_string()))?;

    let snap = session.steps.get(n).ok_or((
        StatusCode::NOT_FOUND,
        format!("Step {n} not found (total: {})", session.step_count()),
    ))?;

    let state_vm = StateViewModel::from_game_state(&snap.state_after, &session.player_names);

    Ok(Json(state_vm))
}

/// `POST /api/load` — load a different script and recompute snapshots.
pub async fn post_load(
    State(state): State<SharedState>,
    Json(req): Json<LoadRequest>,
) -> Result<Json<SessionResponse>, (StatusCode, String)> {
    // Determine the script path.
    // Reject absolute paths to prevent path traversal — scripts must be relative
    // to scripts_dir. Canonicalize and verify the resolved path is under scripts_dir.
    let script_path = {
        let state_r = state.read().await;
        let p = PathBuf::from(&req.path);
        if p.is_absolute() {
            return Err((
                StatusCode::BAD_REQUEST,
                "Path must be relative to scripts_dir, not absolute".to_string(),
            ));
        }
        let joined = state_r.scripts_dir.join(&req.path);
        // Canonicalize to resolve any `..` components and symlinks.
        let canonical = joined.canonicalize().map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                format!("Cannot resolve path {}: {e}", joined.display()),
            )
        })?;
        // Verify the resolved path is under scripts_dir.
        let scripts_dir_canonical = state_r
            .scripts_dir
            .canonicalize()
            .unwrap_or_else(|_| state_r.scripts_dir.clone());
        if !canonical.starts_with(&scripts_dir_canonical) {
            return Err((
                StatusCode::BAD_REQUEST,
                "Path must be within scripts_dir".to_string(),
            ));
        }
        canonical
    };

    // Load and parse the script (blocking I/O — acceptable for a dev tool).
    let json = std::fs::read_to_string(&script_path).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            format!("Cannot read {}: {e}", script_path.display()),
        )
    })?;

    let script: mtg_engine::testing::script_schema::GameScript = serde_json::from_str(&json)
        .map_err(|e| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Cannot parse script: {e}"),
            )
        })?;

    // Build the replay session (runs the whole script through the engine).
    let session = ReplaySession::from_script(&script).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to build replay session: {e}"),
        )
    })?;

    let players: Vec<String> = session.player_map.keys().cloned().collect();
    let response = SessionResponse {
        loaded: true,
        script_id: Some(session.script.metadata.id.clone()),
        script_name: Some(session.script.metadata.name.clone()),
        description: Some(session.script.metadata.description.clone()),
        total_steps: session.step_count(),
        players,
    };

    // Store the new session.
    let mut state_w = state.write().await;
    state_w.session = Some(session);

    Ok(Json(response))
}

/// `GET /api/scripts` — list all available scripts from `scripts_dir`.
pub async fn get_scripts(State(state): State<SharedState>) -> Json<ScriptsResponse> {
    let scripts_dir = state.read().await.scripts_dir.clone();
    let entries = scan_scripts(&scripts_dir);
    let total = entries.len();

    let mut groups: HashMap<String, Vec<ScriptEntry>> = HashMap::new();
    for entry in entries {
        groups
            .entry(entry.subdirectory.clone())
            .or_default()
            .push(entry);
    }

    Json(ScriptsResponse { groups, total })
}

// ── Script scanning ───────────────────────────────────────────────────────────

/// Recursively scan `scripts_dir` for JSON script files.
/// Returns one `ScriptEntry` per file, sorted by path.
fn scan_scripts(scripts_dir: &Path) -> Vec<ScriptEntry> {
    let mut entries = Vec::new();
    scan_dir_recursive(scripts_dir, scripts_dir, &mut entries);
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    entries
}

fn scan_dir_recursive(base: &Path, current: &Path, entries: &mut Vec<ScriptEntry>) {
    let Ok(read_dir) = std::fs::read_dir(current) else {
        return;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir_recursive(base, &path, entries);
        } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(script_entry) = parse_script_entry(base, &path) {
                entries.push(script_entry);
            }
        }
    }
}

/// Parse a script JSON file into a `ScriptEntry` by reading just the metadata.
fn parse_script_entry(base: &Path, path: &Path) -> Option<ScriptEntry> {
    let json = std::fs::read_to_string(path).ok()?;

    // Parse just the metadata fields we need using a partial struct.
    #[derive(Deserialize)]
    struct PartialScript {
        metadata: PartialMetadata,
    }
    #[derive(Deserialize)]
    struct PartialMetadata {
        id: String,
        name: String,
        #[serde(default)]
        description: String,
        #[serde(default)]
        tags: Vec<String>,
        #[serde(default)]
        review_status: String,
    }

    let partial: PartialScript = serde_json::from_str(&json).ok()?;

    // Relative path from scripts_dir.
    let rel_path = path.strip_prefix(base).ok()?;
    let rel_path_str = rel_path.to_string_lossy().to_string();

    // Subdirectory: the first component of the relative path, or "." for root.
    let subdirectory = rel_path
        .components()
        .next()
        .and_then(|c| match c {
            std::path::Component::Normal(s) => {
                let s = s.to_string_lossy().to_string();
                // If this component is the filename itself (no subdir), use "."
                if path.parent() == Some(base) {
                    Some(".".to_string())
                } else {
                    Some(s)
                }
            }
            _ => None,
        })
        .unwrap_or_else(|| ".".to_string());

    Some(ScriptEntry {
        path: rel_path_str,
        id: partial.metadata.id,
        name: partial.metadata.name,
        description: partial.metadata.description,
        tags: partial.metadata.tags,
        review_status: partial.metadata.review_status,
        subdirectory,
    })
}

// ── Conversion helpers ────────────────────────────────────────────────────────

fn assertion_to_view(r: &AssertionResult) -> AssertionResultView {
    AssertionResultView {
        path: r.path.clone(),
        expected: r.expected.clone(),
        actual: r.actual.clone(),
        passed: r.passed,
    }
}

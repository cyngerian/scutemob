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
    /// Review status of the loaded script ("pending_review", "approved", "disputed", "corrected", or "" if none).
    pub review_status: String,
    /// Run result summary for the loaded script (None if no script is loaded).
    pub run_result: Option<RunResult>,
}

/// Summary of running a script through the harness.
#[derive(Debug, Serialize, Clone)]
pub struct RunResult {
    pub passed: bool,
    pub total_assertions: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub first_failure: Option<FailureDetail>,
    pub harness_error: Option<String>,
}

/// Details about the first failed assertion in a run.
#[derive(Debug, Serialize, Clone)]
pub struct FailureDetail {
    pub step_index: usize,
    pub path: String,
    pub expected: serde_json::Value,
    pub actual: serde_json::Value,
}

/// Request body for `POST /api/scripts/run`.
#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub path: String,
}

/// Request body for `POST /api/scripts/approve`.
#[derive(Debug, Deserialize)]
pub struct ApproveRequest {
    pub id: String,
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
            review_status: String::new(),
            run_result: None,
        }),
        Some(session) => {
            // MR-M9.5-11: sort for deterministic API response ordering.
            let mut players: Vec<String> = session.player_map.keys().cloned().collect();
            players.sort();
            let review_status =
                review_status_str(&session.script.metadata.review_status).to_string();
            let run_result = compute_run_result(session);
            Json(SessionResponse {
                loaded: true,
                script_id: Some(session.script.metadata.id.clone()),
                script_name: Some(session.script.metadata.name.clone()),
                description: Some(session.script.metadata.description.clone()),
                total_steps: session.step_count(),
                players,
                review_status,
                run_result: Some(run_result),
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

    // Build the replay session on a dedicated blocking thread with a full stack.
    // Trigger-heavy scripts (prowess, ward) require deep call chains that overflow
    // tokio worker threads (2 MB default stack). spawn_blocking uses a thread with
    // the OS default stack size (typically 8 MB). CR 702.108a / CR 702.21.
    let session = tokio::task::spawn_blocking(move || ReplaySession::from_script(&script))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Replay session panicked: {e}"),
            )
        })?
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to build replay session: {e}"),
            )
        })?;

    // MR-M9.5-11: sort for deterministic API response ordering.
    let mut players: Vec<String> = session.player_map.keys().cloned().collect();
    players.sort();
    let review_status = review_status_str(&session.script.metadata.review_status).to_string();
    let run_result = compute_run_result(&session);
    let response = SessionResponse {
        loaded: true,
        script_id: Some(session.script.metadata.id.clone()),
        script_name: Some(session.script.metadata.name.clone()),
        description: Some(session.script.metadata.description.clone()),
        total_steps: session.step_count(),
        players,
        review_status,
        run_result: Some(run_result),
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

    // Guard: deduplicate by ID, keeping first occurrence and logging conflicts.
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let entries: Vec<ScriptEntry> = entries
        .into_iter()
        .filter(|e| {
            if seen_ids.insert(e.id.clone()) {
                true
            } else {
                eprintln!(
                    "WARNING: duplicate script id '{}' in '{}' — skipping (fix metadata.id)",
                    e.id, e.path
                );
                false
            }
        })
        .collect();

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

// ── New route handlers ────────────────────────────────────────────────────────

/// `POST /api/scripts/run` — run a script through the harness and return a `RunResult`.
/// Does NOT change the currently loaded session.
pub async fn post_run_script(
    State(state): State<SharedState>,
    Json(req): Json<RunRequest>,
) -> Result<Json<RunResult>, (StatusCode, String)> {
    let script_path = resolve_script_path(&state, &req.path).await?;

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

    // Build the replay session on a dedicated blocking thread with a full stack.
    // Prowess and other trigger-heavy scripts overflow tokio worker thread stacks (2 MB).
    let run_result =
        match tokio::task::spawn_blocking(move || ReplaySession::from_script(&script)).await {
            Ok(Ok(session)) => compute_run_result(&session),
            Ok(Err(e)) => RunResult {
                passed: false,
                total_assertions: 0,
                passed_count: 0,
                failed_count: 0,
                first_failure: None,
                harness_error: Some(e.to_string()),
            },
            Err(e) => RunResult {
                passed: false,
                total_assertions: 0,
                passed_count: 0,
                failed_count: 0,
                first_failure: None,
                harness_error: Some(format!("Replay session panicked: {e}")),
            },
        };

    Ok(Json(run_result))
}

/// `POST /api/scripts/approve` — set a script's `review_status` to `"approved"`.
///
/// Scans `scripts_dir` for a JSON file whose `metadata.id` matches, edits it in
/// place, and returns `{ "ok": true }`. Returns 404 if the id is not found.
pub async fn post_approve_script(
    State(state): State<SharedState>,
    Json(req): Json<ApproveRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let scripts_dir = state.read().await.scripts_dir.clone();

    let script_path = find_script_by_id(&scripts_dir, &req.id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("Script id '{}' not found", req.id),
        )
    })?;

    let json = std::fs::read_to_string(&script_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Cannot read {}: {e}", script_path.display()),
        )
    })?;

    let mut value: serde_json::Value = serde_json::from_str(&json).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Cannot parse {}: {e}", script_path.display()),
        )
    })?;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if let Some(meta) = value.get_mut("metadata") {
        meta["review_status"] = serde_json::json!("approved");
        meta["reviewed_by"] = serde_json::json!("stepper");
        meta["review_date"] = serde_json::json!(today);
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Script has no metadata field".to_string(),
        ));
    }

    let out = serde_json::to_string_pretty(&value).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Cannot serialize: {e}"),
        )
    })?;

    std::fs::write(&script_path, format!("{out}\n")).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Cannot write {}: {e}", script_path.display()),
        )
    })?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ── Helper functions ──────────────────────────────────────────────────────────

/// Resolve a relative path string to an absolute, canonicalized path within scripts_dir.
/// Rejects absolute paths and path traversal attempts.
async fn resolve_script_path(
    state: &SharedState,
    path_str: &str,
) -> Result<std::path::PathBuf, (StatusCode, String)> {
    let state_r = state.read().await;
    let p = PathBuf::from(path_str);
    if p.is_absolute() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Path must be relative to scripts_dir, not absolute".to_string(),
        ));
    }
    let joined = state_r.scripts_dir.join(path_str);
    let canonical = joined.canonicalize().map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            format!("Cannot resolve path {}: {e}", joined.display()),
        )
    })?;
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
    Ok(canonical)
}

/// Recursively scan `scripts_dir` for a JSON file whose `metadata.id` matches `target_id`.
fn find_script_by_id(scripts_dir: &Path, target_id: &str) -> Option<PathBuf> {
    find_by_id_recursive(scripts_dir, target_id)
}

fn find_by_id_recursive(dir: &Path, target_id: &str) -> Option<PathBuf> {
    let read_dir = std::fs::read_dir(dir).ok()?;
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_by_id_recursive(&path, target_id) {
                return Some(found);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(id) = read_script_id(&path) {
                if id == target_id {
                    return Some(path);
                }
            }
        }
    }
    None
}

/// Read just the `metadata.id` from a JSON file without full parsing.
fn read_script_id(path: &Path) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct PartialScript {
        metadata: PartialMeta,
    }
    #[derive(serde::Deserialize)]
    struct PartialMeta {
        id: String,
    }
    let json = std::fs::read_to_string(path).ok()?;
    let partial: PartialScript = serde_json::from_str(&json).ok()?;
    Some(partial.metadata.id)
}

/// Compute a `RunResult` from an already-built `ReplaySession`.
pub fn compute_run_result(session: &ReplaySession) -> RunResult {
    let mut total_assertions = 0usize;
    let mut passed_count = 0usize;
    let mut failed_count = 0usize;
    let mut first_failure: Option<FailureDetail> = None;

    for (step_idx, step) in session.steps.iter().enumerate() {
        if let Some(assertions) = &step.assertions {
            for result in assertions {
                total_assertions += 1;
                if result.passed {
                    passed_count += 1;
                } else {
                    failed_count += 1;
                    if first_failure.is_none() {
                        first_failure = Some(FailureDetail {
                            step_index: step_idx,
                            path: result.path.clone(),
                            expected: result.expected.clone(),
                            actual: result.actual.clone(),
                        });
                    }
                }
            }
        }
    }

    RunResult {
        passed: failed_count == 0,
        total_assertions,
        passed_count,
        failed_count,
        first_failure,
        harness_error: None,
    }
}

/// Convert a `ReviewStatus` enum variant to its snake_case string form.
fn review_status_str(status: &mtg_engine::testing::script_schema::ReviewStatus) -> &'static str {
    use mtg_engine::testing::script_schema::ReviewStatus;
    match status {
        ReviewStatus::PendingReview => "pending_review",
        ReviewStatus::Approved => "approved",
        ReviewStatus::Disputed => "disputed",
        ReviewStatus::Corrected => "corrected",
    }
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

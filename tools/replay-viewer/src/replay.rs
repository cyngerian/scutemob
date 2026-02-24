/// Replay engine for the game state stepper.
///
/// Pre-computes all step snapshots from a [`GameScript`] at load time.
/// Stepping through the replay is then a pure O(1) data lookup — no engine
/// re-invocations happen during interactive use.
///
/// # Memory cost
/// im-rs structural sharing means storing N snapshots costs O(changed nodes
/// per step), not O(full state × N). A 100-step script typically uses ~2MB.
use std::collections::HashMap;

use anyhow::Result;
use mtg_engine::testing::script_schema::{GameScript, ScriptAction};
use mtg_engine::{
    build_initial_state, process_command, translate_player_action, Command, GameEvent, GameState,
    PlayerId,
};
use serde::Serialize;

// ── Public types ─────────────────────────────────────────────────────────────

/// A complete replay session for one game script.
/// Stored in axum's shared state behind `Arc<RwLock<...>>`.
#[allow(dead_code)] // fields consumed by API layer in Session 2
pub struct ReplaySession {
    /// The parsed game script.
    pub script: GameScript,
    /// Player name → PlayerId mapping (from `build_initial_state`).
    pub player_map: HashMap<String, PlayerId>,
    /// Reverse map: PlayerId → player name (for view model serialization).
    pub player_names: HashMap<PlayerId, String>,
    /// Pre-computed step snapshots.
    /// Index 0 = initial state before any commands.
    pub steps: Vec<StepSnapshot>,
}

/// One step = one meaningful action from the script + results.
/// Non-command actions (AssertState, informational) are also recorded so the
/// viewer can show every script entry.
#[allow(dead_code)] // fields consumed by API layer in Session 2
pub struct StepSnapshot {
    pub index: usize,
    /// The script action that produced this step.
    pub script_action: ScriptAction,
    /// The engine command that was sent (None for non-command actions).
    pub command: Option<Command>,
    /// All events emitted by this command (empty for non-command steps).
    pub events: Vec<GameEvent>,
    /// Game state after this command resolved (or unchanged for non-command steps).
    pub state_after: GameState,
    /// Assertion results if this step had an `assert_state` action.
    pub assertions: Option<Vec<AssertionResult>>,
}

/// Result of checking one assertion from an `assert_state` action.
#[derive(Debug, Clone, Serialize)]
pub struct AssertionResult {
    pub path: String,
    pub expected: serde_json::Value,
    pub actual: serde_json::Value,
    pub passed: bool,
}

// ── ReplaySession impl ────────────────────────────────────────────────────────

impl ReplaySession {
    /// Pre-compute all step snapshots for a game script.
    ///
    /// This runs the entire script through the engine at load time.
    /// Each `process_command()` call produces a new `GameState` (im-rs clone is O(1)
    /// due to structural sharing), which is stored in the snapshot.
    pub fn from_script(script: &GameScript) -> Result<Self> {
        let (initial_state, player_map) = build_initial_state(&script.initial_state);

        // Build reverse map: PlayerId → name.
        let player_names: HashMap<PlayerId, String> = player_map
            .iter()
            .map(|(name, &pid)| (pid, name.clone()))
            .collect();

        let mut steps: Vec<StepSnapshot> = Vec::new();

        // Step 0: initial state before any commands.
        // We use a synthetic "initial" action to represent this.
        let initial_action = ScriptAction::TurnBasedAction {
            action: "initial_state".to_string(),
            player: None,
            cr_ref: None,
            note: Some("Initial game state before any commands".to_string()),
        };
        steps.push(StepSnapshot {
            index: 0,
            script_action: initial_action,
            command: None,
            events: vec![],
            state_after: initial_state.clone(),
            assertions: None,
        });

        // Track current state as we apply commands.
        let mut current_state = initial_state;

        for (step_idx, script_step) in script.script.iter().enumerate() {
            for (action_idx, action) in script_step.actions.iter().enumerate() {
                let snapshot_index = steps.len();
                match action {
                    ScriptAction::PriorityPass { player, .. } => {
                        if let Some(&pid) = player_map.get(player.as_str()) {
                            let cmd = Command::PassPriority { player: pid };
                            match process_command(current_state.clone(), cmd.clone()) {
                                Ok((new_state, events)) => {
                                    steps.push(StepSnapshot {
                                        index: snapshot_index,
                                        script_action: action.clone(),
                                        command: Some(cmd),
                                        events,
                                        state_after: new_state.clone(),
                                        assertions: None,
                                    });
                                    current_state = new_state;
                                }
                                Err(e) => {
                                    // Record the failed step with error info, keep current state.
                                    steps.push(StepSnapshot {
                                        index: snapshot_index,
                                        script_action: action.clone(),
                                        command: Some(cmd),
                                        events: vec![],
                                        state_after: current_state.clone(),
                                        assertions: Some(vec![AssertionResult {
                                            path: format!("step[{}][{}]", step_idx, action_idx),
                                            expected: serde_json::json!("command_ok"),
                                            actual: serde_json::json!(format!("{e:?}")),
                                            passed: false,
                                        }]),
                                    });
                                }
                            }
                        } else {
                            // Unknown player — record as informational.
                            steps.push(StepSnapshot {
                                index: snapshot_index,
                                script_action: action.clone(),
                                command: None,
                                events: vec![],
                                state_after: current_state.clone(),
                                assertions: None,
                            });
                        }
                    }

                    ScriptAction::PriorityRound {
                        players: round_players,
                        ..
                    } => {
                        // Record each priority pass as a separate snapshot.
                        for pname in round_players {
                            let snap_idx = steps.len();
                            if let Some(&pid) = player_map.get(pname.as_str()) {
                                let cmd = Command::PassPriority { player: pid };
                                match process_command(current_state.clone(), cmd.clone()) {
                                    Ok((new_state, events)) => {
                                        steps.push(StepSnapshot {
                                            index: snap_idx,
                                            script_action: action.clone(),
                                            command: Some(cmd),
                                            events,
                                            state_after: new_state.clone(),
                                            assertions: None,
                                        });
                                        current_state = new_state;
                                    }
                                    Err(e) => {
                                        steps.push(StepSnapshot {
                                            index: snap_idx,
                                            script_action: action.clone(),
                                            command: Some(cmd),
                                            events: vec![],
                                            state_after: current_state.clone(),
                                            assertions: Some(vec![AssertionResult {
                                                path: format!("step[{}][{}]", step_idx, action_idx),
                                                expected: serde_json::json!("command_ok"),
                                                actual: serde_json::json!(format!("{e:?}")),
                                                passed: false,
                                            }]),
                                        });
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    ScriptAction::PlayerAction {
                        player,
                        action: action_str,
                        card,
                        targets,
                        ..
                    } => {
                        if let Some(&pid) = player_map.get(player.as_str()) {
                            let cmd = translate_player_action(
                                action_str.as_str(),
                                pid,
                                card.as_deref(),
                                targets,
                                &current_state,
                                &player_map,
                            );
                            if let Some(cmd) = cmd {
                                match process_command(current_state.clone(), cmd.clone()) {
                                    Ok((new_state, events)) => {
                                        steps.push(StepSnapshot {
                                            index: snapshot_index,
                                            script_action: action.clone(),
                                            command: Some(cmd),
                                            events,
                                            state_after: new_state.clone(),
                                            assertions: None,
                                        });
                                        current_state = new_state;
                                    }
                                    Err(e) => {
                                        steps.push(StepSnapshot {
                                            index: snapshot_index,
                                            script_action: action.clone(),
                                            command: Some(cmd),
                                            events: vec![],
                                            state_after: current_state.clone(),
                                            assertions: Some(vec![AssertionResult {
                                                path: format!("step[{}][{}]", step_idx, action_idx),
                                                expected: serde_json::json!("command_ok"),
                                                actual: serde_json::json!(format!("{e:?}")),
                                                passed: false,
                                            }]),
                                        });
                                    }
                                }
                            } else {
                                // Action not recognized or not translatable — informational.
                                steps.push(StepSnapshot {
                                    index: snapshot_index,
                                    script_action: action.clone(),
                                    command: None,
                                    events: vec![],
                                    state_after: current_state.clone(),
                                    assertions: None,
                                });
                            }
                        } else {
                            steps.push(StepSnapshot {
                                index: snapshot_index,
                                script_action: action.clone(),
                                command: None,
                                events: vec![],
                                state_after: current_state.clone(),
                                assertions: None,
                            });
                        }
                    }

                    ScriptAction::AssertState { assertions, .. } => {
                        // Evaluate assertions against current state.
                        let results = evaluate_assertions(&current_state, assertions, &player_map);
                        steps.push(StepSnapshot {
                            index: snapshot_index,
                            script_action: action.clone(),
                            command: None,
                            events: vec![],
                            state_after: current_state.clone(),
                            assertions: Some(results),
                        });
                    }

                    // Informational actions — no engine command, just record.
                    ScriptAction::StackResolve { .. }
                    | ScriptAction::SbaCheck { .. }
                    | ScriptAction::TriggerPlaced { .. }
                    | ScriptAction::PhaseTransition { .. }
                    | ScriptAction::TurnBasedAction { .. } => {
                        steps.push(StepSnapshot {
                            index: snapshot_index,
                            script_action: action.clone(),
                            command: None,
                            events: vec![],
                            state_after: current_state.clone(),
                            assertions: None,
                        });
                    }
                }
            }
        }

        Ok(ReplaySession {
            script: script.clone(),
            player_map,
            player_names,
            steps,
        })
    }

    /// Returns the total number of steps (including step 0).
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

// ── Assertion evaluation ──────────────────────────────────────────────────────

/// Evaluate dot-notation assertion paths against the current game state.
/// Returns one `AssertionResult` per assertion.
fn evaluate_assertions(
    state: &GameState,
    assertions: &HashMap<String, serde_json::Value>,
    players: &HashMap<String, PlayerId>,
) -> Vec<AssertionResult> {
    use mtg_engine::ZoneId;

    assertions
        .iter()
        .map(|(path, expected)| {
            let parts: Vec<&str> = path.splitn(4, '.').collect();
            let actual = match parts.as_slice() {
                ["players", name, "life"] => players
                    .get(*name)
                    .and_then(|pid| state.players.get(pid))
                    .map(|p| serde_json::json!(p.life_total))
                    .unwrap_or(serde_json::Value::Null),
                ["players", name, "poison_counters"] => players
                    .get(*name)
                    .and_then(|pid| state.players.get(pid))
                    .map(|p| serde_json::json!(p.poison_counters))
                    .unwrap_or(serde_json::Value::Null),
                ["zones", "hand", player, "count"] => {
                    if let Some(&pid) = players.get(*player) {
                        let count = state
                            .objects
                            .values()
                            .filter(|o| o.zone == ZoneId::Hand(pid))
                            .count();
                        serde_json::json!(count)
                    } else {
                        serde_json::Value::Null
                    }
                }
                ["zones", "stack", "count"] => {
                    serde_json::json!(state.stack_objects.len())
                }
                _ => serde_json::Value::Null,
            };

            let passed = &actual == expected;
            AssertionResult {
                path: path.clone(),
                expected: expected.clone(),
                actual,
                passed,
            }
        })
        .collect()
}

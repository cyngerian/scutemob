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
                            let snapshot_cmd = cmd.clone();
                            match process_command(current_state.clone(), cmd) {
                                Ok((new_state, events)) => {
                                    steps.push(StepSnapshot {
                                        index: snapshot_index,
                                        script_action: action.clone(),
                                        command: Some(snapshot_cmd),
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
                                        command: Some(snapshot_cmd),
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
                                let snapshot_cmd = cmd.clone();
                                match process_command(current_state.clone(), cmd) {
                                    Ok((new_state, events)) => {
                                        steps.push(StepSnapshot {
                                            index: snap_idx,
                                            script_action: action.clone(),
                                            command: Some(snapshot_cmd),
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
                                            command: Some(snapshot_cmd),
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
                        ability_index,
                        attackers,
                        blockers,
                        convoke,
                        delve,
                        kicked,
                        ..
                    } => {
                        if let Some(&pid) = player_map.get(player.as_str()) {
                            let cmd = translate_player_action(
                                action_str.as_str(),
                                pid,
                                card.as_deref(),
                                *ability_index as usize,
                                targets,
                                attackers,
                                blockers,
                                convoke,
                                delve,
                                *kicked,
                                &current_state,
                                &player_map,
                            );
                            if let Some(cmd) = cmd {
                                let snapshot_cmd = cmd.clone();
                                match process_command(current_state.clone(), cmd) {
                                    Ok((new_state, events)) => {
                                        steps.push(StepSnapshot {
                                            index: snapshot_index,
                                            script_action: action.clone(),
                                            command: Some(snapshot_cmd),
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
                                            command: Some(snapshot_cmd),
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use mtg_engine::testing::script_schema::GameScript;

    /// Load a game script from test-data relative to the workspace root.
    /// Tests run from the package directory (tools/replay-viewer/), so we
    /// walk up two levels to reach the workspace root.
    fn load_baseline_script(filename: &str) -> GameScript {
        let path = format!("../../test-data/generated-scripts/baseline/{}", filename);
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Failed to read script at {path}"));
        serde_json::from_str(&json).unwrap_or_else(|e| panic!("Failed to parse {filename}: {e}"))
    }

    #[test]
    fn test_from_script_produces_steps() {
        // 001: 2-player priority pass — 1 initial state + 4 priority passes = 5 steps.
        let script = load_baseline_script("001_priority_pass_empty_stack.json");
        let session = ReplaySession::from_script(&script).unwrap();
        assert_eq!(
            session.step_count(),
            5,
            "expected 5 steps (1 initial + 4 priority passes)"
        );
    }

    #[test]
    fn test_step_zero_is_initial_state() {
        let script = load_baseline_script("001_priority_pass_empty_stack.json");
        let session = ReplaySession::from_script(&script).unwrap();
        let step0 = &session.steps[0];
        assert_eq!(step0.index, 0);
        assert!(step0.command.is_none(), "step 0 must have no command");
        assert!(step0.events.is_empty(), "step 0 must have no events");
        assert!(step0.assertions.is_none(), "step 0 must have no assertions");
    }

    #[test]
    fn test_initial_life_totals_are_40() {
        // Commander starts at 40 life.
        let script = load_baseline_script("001_priority_pass_empty_stack.json");
        let session = ReplaySession::from_script(&script).unwrap();
        for (_pid, player) in &session.steps[0].state_after.players {
            assert_eq!(
                player.life_total, 40,
                "all players start at 40 life in Commander"
            );
        }
    }

    #[test]
    fn test_player_map_reverse_map_match() {
        let script = load_baseline_script("001_priority_pass_empty_stack.json");
        let session = ReplaySession::from_script(&script).unwrap();
        assert!(
            !session.player_map.is_empty(),
            "player_map must be populated"
        );
        assert_eq!(
            session.player_map.len(),
            session.player_names.len(),
            "player_map and player_names must have the same cardinality"
        );
        // Every entry in player_map must round-trip through player_names.
        for (name, pid) in &session.player_map {
            assert_eq!(
                session.player_names.get(pid),
                Some(name),
                "player_names[{pid:?}] must equal '{name}'"
            );
        }
    }

    #[test]
    fn test_some_steps_have_commands() {
        let script = load_baseline_script("001_priority_pass_empty_stack.json");
        let session = ReplaySession::from_script(&script).unwrap();
        // Not every step has a command (informational actions like PhaseTransition
        // also produce steps with command: None), but at least one step after the
        // initial state must carry a command (a priority pass).
        let command_count = session.steps.iter().filter(|s| s.command.is_some()).count();
        assert!(
            command_count >= 1,
            "must have at least one step with a command"
        );
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
            // Use splitn(5) to handle up to 4-segment paths like
            // players.p2.commander_damage_received.p1
            let parts: Vec<&str> = path.splitn(5, '.').collect();
            let (actual, passed) = match parts.as_slice() {
                // ── Player stats ──────────────────────────────────────────────
                ["players", name, "life"] => {
                    let v = players
                        .get(*name)
                        .and_then(|pid| state.players.get(pid))
                        .map(|p| serde_json::json!(p.life_total))
                        .unwrap_or(serde_json::Value::Null);
                    let ok = &v == expected;
                    (v, ok)
                }

                ["players", name, "poison_counters"] => {
                    let v = players
                        .get(*name)
                        .and_then(|pid| state.players.get(pid))
                        .map(|p| serde_json::json!(p.poison_counters))
                        .unwrap_or(serde_json::Value::Null);
                    let ok = &v == expected;
                    (v, ok)
                }

                // commander_damage_received is keyed by source PlayerId → CardId → u32.
                // The assertion checks total damage from one player's commanders to another.
                ["players", target, "commander_damage_received", source] => {
                    let total = players
                        .get(*target)
                        .and_then(|tpid| state.players.get(tpid))
                        .and_then(|p| {
                            players.get(*source).map(|spid| {
                                p.commander_damage_received
                                    .get(spid)
                                    .map(|by_card| by_card.values().sum::<u32>())
                                    .unwrap_or(0)
                            })
                        })
                        .unwrap_or(0);
                    let v = serde_json::json!(total);
                    let ok = &v == expected;
                    (v, ok)
                }

                // ── Zone counts ───────────────────────────────────────────────
                ["zones", "hand", player, "count"] => {
                    if let Some(&pid) = players.get(*player) {
                        let count = state
                            .objects
                            .values()
                            .filter(|o| o.zone == ZoneId::Hand(pid))
                            .count();
                        let v = serde_json::json!(count);
                        let ok = &v == expected;
                        (v, ok)
                    } else {
                        (serde_json::Value::Null, false)
                    }
                }

                ["zones", "stack", "count"] => {
                    let v = serde_json::json!(state.stack_objects.len());
                    let ok = &v == expected;
                    (v, ok)
                }

                // ── Zone membership (includes/excludes/is_empty) ──────────────
                ["zones", "stack"] => {
                    let is_empty = state.stack_objects.is_empty();
                    let count = state.stack_objects.len();
                    let v = serde_json::json!({ "count": count, "is_empty": is_empty });
                    let ok = check_stack_assertion(is_empty, count, expected);
                    (v, ok)
                }

                // Battlefield is shared — filter by controller name.
                ["zones", "battlefield", player] => {
                    let names = if let Some(&pid) = players.get(*player) {
                        state
                            .objects
                            .values()
                            .filter(|o| o.zone == ZoneId::Battlefield && o.controller == pid)
                            .map(|o| o.characteristics.name.clone())
                            .collect::<Vec<_>>()
                    } else {
                        vec![]
                    };
                    let ok = check_list_assertion(&names, expected);
                    (serde_json::json!(names), ok)
                }

                ["zones", "graveyard", player] => {
                    let names = if let Some(&pid) = players.get(*player) {
                        state
                            .objects
                            .values()
                            .filter(|o| o.zone == ZoneId::Graveyard(pid))
                            .map(|o| o.characteristics.name.clone())
                            .collect::<Vec<_>>()
                    } else {
                        vec![]
                    };
                    let ok = check_list_assertion(&names, expected);
                    (serde_json::json!(names), ok)
                }

                // Exile is a shared zone (no per-player ZoneId).
                ["zones", "exile"] => {
                    let names: Vec<String> = state
                        .objects
                        .values()
                        .filter(|o| o.zone == ZoneId::Exile)
                        .map(|o| o.characteristics.name.clone())
                        .collect();
                    let ok = check_list_assertion(&names, expected);
                    (serde_json::json!(names), ok)
                }

                ["zones", "library", player, "count"] => {
                    if let Some(&pid) = players.get(*player) {
                        let count = state
                            .objects
                            .values()
                            .filter(|o| o.zone == ZoneId::Library(pid))
                            .count();
                        let v = serde_json::json!(count);
                        let ok = &v == expected;
                        (v, ok)
                    } else {
                        (serde_json::Value::Null, false)
                    }
                }

                // `permanent.<CardName>.tapped` — check tapped state of a named battlefield permanent.
                // Card names may contain spaces; splitn(5, '.') preserves spaces in the name segment.
                ["permanent", name, "tapped"] => {
                    let v = state
                        .objects
                        .values()
                        .find(|o| o.characteristics.name == *name && o.zone == ZoneId::Battlefield)
                        .map(|o| serde_json::json!(o.status.tapped))
                        .unwrap_or(serde_json::Value::Null);
                    let ok = &v == expected;
                    (v, ok)
                }

                _ => (serde_json::Value::Null, false),
            };

            AssertionResult {
                path: path.clone(),
                expected: expected.clone(),
                actual,
                passed,
            }
        })
        .collect()
}

/// Check `{"is_empty": bool}` or `{"count": N}` assertions against the stack.
fn check_stack_assertion(is_empty: bool, count: usize, expected: &serde_json::Value) -> bool {
    if let Some(obj) = expected.as_object() {
        if let Some(exp) = obj.get("is_empty").and_then(|v| v.as_bool()) {
            return is_empty == exp;
        }
        if let Some(exp) = obj.get("count").and_then(|v| v.as_u64()) {
            return count as u64 == exp;
        }
    }
    // Fallback: direct equality with count.
    &serde_json::json!(count) == expected
}

/// Check `{"includes": [...], "excludes": [...]}` assertions against a list of card names.
/// Each item in includes/excludes can be a plain string or `{"card": "Name"}`.
fn check_list_assertion(names: &[String], expected: &serde_json::Value) -> bool {
    let Some(obj) = expected.as_object() else {
        return false;
    };

    let card_name = |item: &serde_json::Value| -> Option<String> {
        item.as_str().map(|s| s.to_string()).or_else(|| {
            item.get("card")
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
        })
    };

    let includes_ok = obj
        .get("includes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .all(|item| card_name(item).is_some_and(|n| names.iter().any(|a| a == &n)))
        })
        .unwrap_or(true);

    let excludes_ok = obj
        .get("excludes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .all(|item| card_name(item).is_none_or(|n| !names.iter().any(|a| a == &n)))
        })
        .unwrap_or(true);

    includes_ok && excludes_ok
}

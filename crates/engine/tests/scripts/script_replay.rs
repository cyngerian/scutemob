// Game Script Replay Harness — Hook 2 from `docs/mtg-engine-game-scripts.md`.
//
// Reads a [`GameScript`] JSON file, constructs the initial game state via
// [`GameStateBuilder`], feeds each [`ScriptAction`] into the engine as a [`Command`],
// and asserts game state at every `assert_state` checkpoint.
//
// ## Design
//
// - Player names in scripts (e.g. `"p1"`, `"alice"`) are sorted alphabetically and
//   mapped to `PlayerId(1)`, `PlayerId(2)`, … in that order.
// - Card names are resolved to `ObjectId` by scanning `state.objects()`.
// - Assertions use dot-notation paths into game state (see `check_assertions`).
//
// ## Nothing is skipped silently (SR-9c)
//
// Three holes were closed here. Each one let a script look green while asserting
// less than it claimed:
//
// 1. **An unrecognized assertion path used to return `None`** — i.e. "no mismatch".
//    244 of the corpus's 3,161 assertions were spelled in paths `check_assertions`
//    did not implement (`zones.battlefield.<p>.count`, `zones.exile`,
//    `zones.hand.<p>`, `permanent.<c>.power`, …) and were therefore *not checked*.
//    Every path the corpus uses is now implemented, and an unknown path is a hard
//    [`AssertionMismatch`], not a pass.
// 2. **A `player_action` the harness could not translate used to leave the state
//    untouched and say nothing.** It now yields [`ReplayResult::ActionNotTranslated`],
//    which `run_all_scripts` accounts for against a shrinking allowlist.
// 3. **A player name that is not in the script's own `players` map** used to be
//    ignored. It is now [`ReplayResult::UnknownPlayer`].
//
// ## Supported assertion paths
//
// | Path | Type |
// |------|------|
// | `players.<name>.life` | `i32` exact match |
// | `players.<name>.poison_counters` | `u32` exact match |
// | `players.<name>.has_citys_blessing` | `bool` exact match |
// | `players.<name>.mana_pool.<color>` | `u32` exact match |
// | `players.<name>.commander_damage_received.<from_player>` | `u32` exact match (summed over that player's commanders) |
// | `zones.hand.<player>.count` | `usize` exact match |
// | `zones.hand.<player>` | includes/excludes card name list |
// | `zones.graveyard.<player>` | includes/excludes card name list |
// | `zones.graveyard.<player>.count` | `usize` exact match |
// | `zones.battlefield.<player>` | includes/excludes card name list |
// | `zones.battlefield.<player>.count` | `usize` exact match |
// | `zones.battlefield` | `is_empty` check (all controllers) |
// | `zones.library.<player>.count` | `usize` exact match |
// | `zones.exile` | includes/excludes card name list |
// | `zones.exile.count` | `usize` exact match |
// | `zones.stack.count` | `usize` exact match |
// | `zones.stack` | `is_empty` check |
// | `permanent.<card_name>.tapped` | `bool` exact match |
// | `permanent.<card_name>.power` | `i32` exact match (post-layers) |
// | `permanent.<card_name>.toughness` | `i32` exact match (post-layers) |
// | `permanent.<card_name>.counters.<type>` | `u32` exact match |

use std::collections::HashMap;

use mtg_engine::testing::replay_harness::{
    build_initial_state, parse_counter_type, translate_player_action,
};
use mtg_engine::testing::script_schema::{GameScript, ScriptAction};
use mtg_engine::{
    calculate_characteristics, process_command, Command, GameState, PlayerId, ZoneId,
};

// ── Public API ────────────────────────────────────────────────────────────────

/// Describes a single assertion failure.
///
/// The fields are consumed by `run_all_scripts` (a sibling module in the same
/// `scripts` test binary) when it formats failures; within *this* module only
/// the constructors run, so the fields read as unused. Same reason
/// `ReplayResult` below carries the attribute.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AssertionMismatch {
    pub path: String,
    pub expected: serde_json::Value,
    pub actual: String,
}

/// Outcome of replaying one `assert_state` checkpoint.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ReplayResult {
    /// All assertions at this checkpoint matched.
    Ok {
        step_idx: usize,
        action_idx: usize,
        description: String,
    },
    /// One or more assertions failed.
    Mismatch {
        step_idx: usize,
        action_idx: usize,
        description: String,
        mismatches: Vec<AssertionMismatch>,
    },
    /// The engine rejected a command from this checkpoint.
    CommandRejected {
        step_idx: usize,
        action_idx: usize,
        error: String,
    },
    /// A `player_action` whose `action` string `translate_player_action` does not
    /// map to any `Command`. The action ran as a no-op: the game state is exactly
    /// what it was before, and every assertion downstream of it is describing a
    /// board the script never actually reached.
    ///
    /// Before SR-9c this was `state_slot = Some(state)` with no record at all.
    ActionNotTranslated {
        step_idx: usize,
        action_idx: usize,
        action: String,
    },
    /// An action naming a player that the script's own `initial_state.players`
    /// map does not contain. Also previously a silent no-op.
    UnknownPlayer {
        step_idx: usize,
        action_idx: usize,
        player: String,
    },
}

/// Replay a complete [`GameScript`] through the engine.
///
/// Returns one [`ReplayResult`] per `assert_state` checkpoint encountered.
/// Command rejections are also recorded as [`ReplayResult::CommandRejected`].
pub fn replay_script(script: &GameScript) -> Vec<ReplayResult> {
    let (state, players) = build_initial_state(&script.initial_state);
    // Wrap state in Option so we can safely take ownership then put it back.
    let mut state_slot: Option<GameState> = Some(state);
    let mut results = Vec::new();

    'outer: for (step_idx, step) in script.script.iter().enumerate() {
        for (action_idx, action) in step.actions.iter().enumerate() {
            // If state is gone (prior command rejected, state consumed), stop.
            let Some(state) = state_slot.take() else {
                break 'outer;
            };

            match action {
                ScriptAction::PriorityPass { player, .. } => {
                    if let Some(&pid) = players.get(player.as_str()) {
                        match process_command(state, Command::PassPriority { player: pid }) {
                            Ok((s, _)) => state_slot = Some(s),
                            Err(e) => {
                                results.push(ReplayResult::CommandRejected {
                                    step_idx,
                                    action_idx,
                                    error: format!("{e:?}"),
                                });
                                // state_slot remains None; loop exits at top of next iteration.
                            }
                        }
                    } else {
                        results.push(ReplayResult::UnknownPlayer {
                            step_idx,
                            action_idx,
                            player: player.clone(),
                        });
                        state_slot = Some(state);
                    }
                }

                ScriptAction::PriorityRound {
                    players: round_players,
                    ..
                } => {
                    // Use Option<GameState> inside the sub-loop to handle ownership correctly.
                    let mut current: Option<GameState> = Some(state);
                    for pname in round_players {
                        let Some(s) = current.take() else { break };
                        if let Some(&pid) = players.get(pname.as_str()) {
                            match process_command(s, Command::PassPriority { player: pid }) {
                                Ok((new_s, _)) => current = Some(new_s),
                                Err(e) => {
                                    results.push(ReplayResult::CommandRejected {
                                        step_idx,
                                        action_idx,
                                        error: format!("{e:?}"),
                                    });
                                    // current remains None → state_slot stays None.
                                }
                            }
                        } else {
                            results.push(ReplayResult::UnknownPlayer {
                                step_idx,
                                action_idx,
                                player: pname.clone(),
                            });
                            current = Some(s);
                        }
                    }
                    state_slot = current; // None if a command failed, Some otherwise.
                }

                ScriptAction::PlayerAction {
                    player,
                    action,
                    card,
                    targets,
                    ability_index,
                    attackers,
                    blockers,
                    convoke,
                    improvise,
                    delve,
                    escape,
                    kicked,
                    buyback,
                    enlist,
                    attacker_name,
                    discard_land,
                    discard_card,
                    bargain_sacrifice,
                    emerge_sacrifice,
                    casualty_sacrifice,
                    assist_player,
                    assist_amount,
                    replicate_count,
                    splice_card_names,
                    escalate_modes,
                    modes,
                    target_creature,
                    x_value,
                    collect_evidence_cards,
                    squad_count,
                    mutate_on_top,
                    gift_opponent,
                    sacrifice_card,
                    exert,
                    pitch_exile_card,
                    chosen_color,
                    hybrid_choices,
                    phyrexian_life_payments,
                    ..
                } => {
                    if let Some(&pid) = players.get(player.as_str()) {
                        let cmd = translate_player_action(
                            action.as_str(),
                            pid,
                            card.as_deref(),
                            *ability_index as usize,
                            targets,
                            attackers,
                            blockers,
                            convoke,
                            improvise,
                            delve,
                            escape,
                            *kicked,
                            *buyback,
                            enlist,
                            attacker_name.as_deref(),
                            discard_land.as_deref(),
                            discard_card.as_deref(),
                            bargain_sacrifice.as_deref(),
                            emerge_sacrifice.as_deref(),
                            casualty_sacrifice.as_deref(),
                            assist_player.as_deref(),
                            *assist_amount,
                            *replicate_count,
                            splice_card_names,
                            *escalate_modes,
                            modes.clone(),
                            target_creature.as_deref(),
                            *x_value,
                            collect_evidence_cards,
                            *squad_count,
                            *mutate_on_top,
                            gift_opponent.as_deref(),
                            sacrifice_card.as_deref(),
                            exert,
                            pitch_exile_card.as_deref(),
                            chosen_color.as_deref(),
                            hybrid_choices,
                            phyrexian_life_payments,
                            &state,
                            &players,
                        );
                        if let Some(cmd) = cmd {
                            match process_command(state, cmd) {
                                Ok((s, _)) => state_slot = Some(s),
                                Err(e) => {
                                    results.push(ReplayResult::CommandRejected {
                                        step_idx,
                                        action_idx,
                                        error: format!("{e:?}"),
                                    });
                                }
                            }
                        } else {
                            results.push(ReplayResult::ActionNotTranslated {
                                step_idx,
                                action_idx,
                                action: action.clone(),
                            });
                            state_slot = Some(state);
                        }
                    } else {
                        results.push(ReplayResult::UnknownPlayer {
                            step_idx,
                            action_idx,
                            player: player.clone(),
                        });
                        state_slot = Some(state);
                    }
                }

                ScriptAction::AssertState {
                    description,
                    assertions,
                    ..
                } => {
                    let mismatches = check_assertions(&state, assertions, &players);
                    if mismatches.is_empty() {
                        results.push(ReplayResult::Ok {
                            step_idx,
                            action_idx,
                            description: description.clone(),
                        });
                    } else {
                        results.push(ReplayResult::Mismatch {
                            step_idx,
                            action_idx,
                            description: description.clone(),
                            mismatches,
                        });
                    }
                    state_slot = Some(state); // put state back after assertion
                }

                // Informational actions — no engine command needed.
                // `TurnBasedAction.action` is intentionally not read: every
                // turn-based action is informational here (see the empty-string
                // contract on `ScriptAction::TurnBasedAction`).
                ScriptAction::StackResolve { .. }
                | ScriptAction::SbaCheck { .. }
                | ScriptAction::TriggerPlaced { .. }
                | ScriptAction::PhaseTransition { .. }
                | ScriptAction::TurnBasedAction { .. } => {
                    state_slot = Some(state);
                }
            }
        }
    }

    results
}

// ── Assertion checker ─────────────────────────────────────────────────────────

/// Evaluate dot-notation assertion paths against the current game state.
///
/// Returns a list of mismatches (empty if all assertions pass).
pub fn check_assertions(
    state: &GameState,
    assertions: &HashMap<String, serde_json::Value>,
    players: &HashMap<String, PlayerId>,
) -> Vec<AssertionMismatch> {
    let mut mismatches = Vec::new();

    for (path, expected) in assertions {
        let parts: Vec<&str> = path.splitn(4, '.').collect();

        let mismatch = match parts.as_slice() {
            // players.<name>.life
            ["players", name, "life"] => {
                if let Some(&pid) = players.get(*name) {
                    let actual = state.players().get(&pid).map(|p| p.life_total).unwrap_or(0);
                    check_exact(path, expected, serde_json::json!(actual))
                } else {
                    Some(AssertionMismatch {
                        path: path.clone(),
                        expected: expected.clone(),
                        actual: format!("player '{}' not found", name),
                    })
                }
            }

            // players.<name>.poison_counters
            ["players", name, "poison_counters"] => {
                if let Some(&pid) = players.get(*name) {
                    let actual = state
                        .players()
                        .get(&pid)
                        .map(|p| p.poison_counters)
                        .unwrap_or(0);
                    check_exact(path, expected, serde_json::json!(actual))
                } else {
                    Some(AssertionMismatch {
                        path: path.clone(),
                        expected: expected.clone(),
                        actual: format!("player '{}' not found", name),
                    })
                }
            }

            // players.<name>.has_citys_blessing (CR 702.131a)
            ["players", name, "has_citys_blessing"] => {
                with_player(path, expected, players, name, |pid| {
                    let actual = state
                        .players()
                        .get(&pid)
                        .map(|p| p.has_citys_blessing)
                        .unwrap_or(false);
                    check_exact(path, expected, serde_json::json!(actual))
                })
            }

            // players.<name>.mana_pool.<color>
            ["players", name, "mana_pool", color] => {
                with_player(path, expected, players, name, |pid| {
                    let Some(p) = state.players().get(&pid) else {
                        return mismatch(path, expected, "player state missing".to_string());
                    };
                    let pool = &p.mana_pool;
                    let actual = match *color {
                        "white" | "w" => pool.white,
                        "blue" | "u" => pool.blue,
                        "black" | "b" => pool.black,
                        "red" | "r" => pool.red,
                        "green" | "g" => pool.green,
                        "colorless" | "c" => pool.colorless,
                        "total" => pool.total(),
                        other => {
                            return mismatch(
                                path,
                                expected,
                                format!("unknown mana color '{}'", other),
                            )
                        }
                    };
                    check_exact(path, expected, serde_json::json!(actual))
                })
            }

            // players.<name>.commander_damage_received.<from_player> (CR 903.10a)
            //
            // Summed across every commander that `from_player` owns: the script
            // schema names a *player*, the engine keys by `(PlayerId, CardId)`.
            ["players", name, "commander_damage_received", from] => {
                with_player(path, expected, players, name, |pid| {
                    let Some(&from_pid) = players.get(*from) else {
                        return mismatch(path, expected, format!("player '{}' not found", from));
                    };
                    let actual: u32 = state
                        .players()
                        .get(&pid)
                        .and_then(|p| p.commander_damage_received.get(&from_pid))
                        .map(|per_cmdr| per_cmdr.values().sum())
                        .unwrap_or(0);
                    check_exact(path, expected, serde_json::json!(actual))
                })
            }

            // zones.hand.<player>.count
            ["zones", "hand", player, "count"] => {
                with_player(path, expected, players, player, |pid| {
                    let count = count_in_zone(state, ZoneId::Hand(pid));
                    check_exact(path, expected, serde_json::json!(count))
                })
            }

            // zones.hand.<player> — includes/excludes card names
            ["zones", "hand", player] => with_player(path, expected, players, player, |pid| {
                let names = names_in_zone(state, ZoneId::Hand(pid));
                check_list_value(path, expected, &names, |n| names.contains(n))
            }),

            // zones.library.<player>.count
            ["zones", "library", player, "count"] => {
                with_player(path, expected, players, player, |pid| {
                    let count = count_in_zone(state, ZoneId::Library(pid));
                    check_exact(path, expected, serde_json::json!(count))
                })
            }

            // zones.stack.count
            ["zones", "stack", "count"] => {
                let count = state.stack_objects().len();
                check_exact(path, expected, serde_json::json!(count))
            }

            // zones.stack — `{"is_empty": bool}`.
            //
            // SR-9c: this used to be `check_list_value(path, expected, &[], |_| false)` —
            // an *empty* name list, unconditionally. `names.is_empty()` was therefore
            // always `true`, so all 583 `"zones.stack": {"is_empty": true}` assertions in
            // the corpus passed without ever looking at the stack. `check_exact` treats a
            // scalar `0` as empty, so this handles both `{"is_empty": …}` and a bare count.
            ["zones", "stack"] => check_exact(
                path,
                expected,
                serde_json::json!(state.stack_objects().len()),
            ),

            // zones.exile — includes/excludes card names
            ["zones", "exile"] => {
                let names = names_in_zone(state, ZoneId::Exile);
                check_list_value(path, expected, &names, |n| names.contains(n))
            }

            // zones.exile.count
            ["zones", "exile", "count"] => {
                let count = count_in_zone(state, ZoneId::Exile);
                check_exact(path, expected, serde_json::json!(count))
            }

            // zones.graveyard.<player> — includes/excludes card names
            ["zones", "graveyard", player] => with_player(path, expected, players, player, |pid| {
                let names = names_in_zone(state, ZoneId::Graveyard(pid));
                check_list_value(path, expected, &names, |n| names.contains(n))
            }),

            // zones.graveyard.<player>.count
            ["zones", "graveyard", player, "count"] => {
                with_player(path, expected, players, player, |pid| {
                    let count = count_in_zone(state, ZoneId::Graveyard(pid));
                    check_exact(path, expected, serde_json::json!(count))
                })
            }

            // zones.battlefield — is_empty across all controllers
            ["zones", "battlefield"] => {
                let names = names_in_zone(state, ZoneId::Battlefield);
                check_list_value(path, expected, &names, |n| names.contains(n))
            }

            // zones.battlefield.<player> — includes/excludes card names
            ["zones", "battlefield", player] => {
                with_player(path, expected, players, player, |pid| {
                    let names = battlefield_names(state, pid);
                    check_list_value(path, expected, &names, |n| names.contains(n))
                })
            }

            // zones.battlefield.<player>.count
            ["zones", "battlefield", player, "count"] => {
                with_player(path, expected, players, player, |pid| {
                    let count = battlefield_names(state, pid).len();
                    check_exact(path, expected, serde_json::json!(count))
                })
            }

            // permanent.<card_name>.tapped
            ["permanent", card_name, "tapped"] => {
                with_permanent(state, path, expected, card_name, |obj, _| {
                    check_exact(path, expected, serde_json::json!(obj.status.tapped))
                })
            }

            // permanent.<card_name>.power / .toughness
            //
            // Read through `calculate_characteristics`, never `obj.characteristics`:
            // on the battlefield the printed value is pre-layers and a script asserting
            // a pumped creature's power off the raw field would be asserting the wrong
            // number (W3-LC layer audit).
            ["permanent", card_name, field @ ("power" | "toughness")] => {
                with_permanent(state, path, expected, card_name, |_, id| {
                    let Some(chars) = calculate_characteristics(state, id) else {
                        return mismatch(
                            path,
                            expected,
                            format!("'{}' has no calculable characteristics", card_name),
                        );
                    };
                    let actual = if *field == "power" {
                        chars.power
                    } else {
                        chars.toughness
                    };
                    match actual {
                        Some(v) => check_exact(path, expected, serde_json::json!(v)),
                        None => mismatch(
                            path,
                            expected,
                            format!("'{}' has no {} (not a creature?)", card_name, field),
                        ),
                    }
                })
            }

            // permanent.<card_name>.counters.<type>
            ["permanent", card_name, "counters", ctype] => {
                with_permanent(state, path, expected, card_name, |obj, _| {
                    let Some(ct) = parse_counter_type(ctype) else {
                        return mismatch(path, expected, format!("unknown counter '{}'", ctype));
                    };
                    let count = obj.counters.get(&ct).copied().unwrap_or(0);
                    check_exact(path, expected, serde_json::json!(count))
                })
            }

            // SR-9c: an unrecognized path used to return `None` — "no mismatch" — so a
            // script could assert anything it liked in a path this function had never
            // heard of and go green. 244 assertions across the corpus were doing exactly
            // that. An unknown path is now a failure; add the path above to fix it.
            _ => mismatch(
                path,
                expected,
                "unsupported assertion path (see script_replay.rs for the supported set)"
                    .to_string(),
            ),
        };

        if let Some(m) = mismatch {
            mismatches.push(m);
        }
    }

    mismatches
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object_on_battlefield<'a>(
    state: &'a GameState,
    name: &str,
) -> Option<(&'a mtg_engine::state::GameObject, mtg_engine::ObjectId)> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, obj)| (obj, *id))
}

/// Build an [`AssertionMismatch`]. Always `Some` — the `Option` is the caller's
/// "did this assertion fail" channel, and every use of this helper means "yes".
fn mismatch(path: &str, expected: &serde_json::Value, actual: String) -> Option<AssertionMismatch> {
    Some(AssertionMismatch {
        path: path.to_string(),
        expected: expected.clone(),
        actual,
    })
}

/// Resolve a script player name, or fail the assertion.
fn with_player(
    path: &str,
    expected: &serde_json::Value,
    players: &HashMap<String, PlayerId>,
    name: &str,
    f: impl FnOnce(PlayerId) -> Option<AssertionMismatch>,
) -> Option<AssertionMismatch> {
    match players.get(name) {
        Some(&pid) => f(pid),
        None => mismatch(path, expected, format!("player '{}' not found", name)),
    }
}

/// Resolve a battlefield permanent by name, or fail the assertion.
fn with_permanent(
    state: &GameState,
    path: &str,
    expected: &serde_json::Value,
    card_name: &str,
    f: impl FnOnce(&mtg_engine::state::GameObject, mtg_engine::ObjectId) -> Option<AssertionMismatch>,
) -> Option<AssertionMismatch> {
    match find_object_on_battlefield(state, card_name) {
        Some((obj, id)) => f(obj, id),
        None => mismatch(
            path,
            expected,
            format!("permanent '{}' not found on battlefield", card_name),
        ),
    }
}

fn names_in_zone(state: &GameState, zone: ZoneId) -> Vec<String> {
    state
        .objects()
        .values()
        .filter(|o| o.zone == zone)
        .map(|o| o.characteristics.name.clone())
        .collect()
}

fn count_in_zone(state: &GameState, zone: ZoneId) -> usize {
    state.objects().values().filter(|o| o.zone == zone).count()
}

fn battlefield_names(state: &GameState, pid: PlayerId) -> Vec<String> {
    state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.controller == pid)
        .map(|o| o.characteristics.name.clone())
        .collect()
}

/// Check an exact-match or comparison assertion.
fn check_exact(
    path: &str,
    expected: &serde_json::Value,
    actual: serde_json::Value,
) -> Option<AssertionMismatch> {
    // Support compound assertion objects.
    if let Some(obj) = expected.as_object() {
        if let Some(less_than) = obj.get("less_than").and_then(|v| v.as_i64()) {
            let actual_i = actual.as_i64().unwrap_or(i64::MAX);
            if actual_i < less_than {
                return None;
            }
            return Some(AssertionMismatch {
                path: path.to_string(),
                expected: expected.clone(),
                actual: actual.to_string(),
            });
        }
        if let Some(is_empty) = obj.get("is_empty").and_then(|v| v.as_bool()) {
            // For scalars, treat 0 == empty.
            let actually_empty =
                matches!(actual, serde_json::Value::Null) || actual.as_i64() == Some(0);
            if actually_empty == is_empty {
                return None;
            }
            return Some(AssertionMismatch {
                path: path.to_string(),
                expected: expected.clone(),
                actual: actual.to_string(),
            });
        }
    }

    if &actual == expected {
        None
    } else {
        Some(AssertionMismatch {
            path: path.to_string(),
            expected: expected.clone(),
            actual: actual.to_string(),
        })
    }
}

/// Read a card name out of an `includes`/`excludes` entry: either the bare string
/// `"Sol Ring"` or the object `{"card": "Sol Ring"}`. Both spellings appear in the
/// corpus, in both list positions.
fn entry_card_name(item: &serde_json::Value) -> Option<&str> {
    item.as_str().or_else(|| {
        item.as_object()
            .and_then(|o| o.get("card"))
            .and_then(|v| v.as_str())
    })
}

/// Check a list-based assertion (includes / excludes / is_empty) against a
/// collection of card names.
///
/// SR-9c changed three things. `includes` no longer `return`s before `excludes` is
/// looked at (six assertions in the corpus carry both, and the `excludes` half was
/// dead). A malformed entry — one that is neither a string nor `{"card": …}` — used
/// to silently become the empty name `""` and then "not be found"; it is now its own
/// diagnostic. And an `expected` shape this function does not understand is a
/// mismatch rather than a pass.
fn check_list_value(
    path: &str,
    expected: &serde_json::Value,
    names: &[String],
    contains: impl Fn(&String) -> bool,
) -> Option<AssertionMismatch> {
    if let Some(obj) = expected.as_object() {
        let mut understood = false;

        if let Some(is_empty) = obj.get("is_empty").and_then(|v| v.as_bool()) {
            understood = true;
            if names.is_empty() != is_empty {
                return mismatch(path, expected, format!("{:?}", names));
            }
        }

        if let Some(includes) = obj.get("includes").and_then(|v| v.as_array()) {
            understood = true;
            for item in includes {
                let Some(card_name) = entry_card_name(item) else {
                    return mismatch(path, expected, format!("malformed includes entry {}", item));
                };
                if !contains(&card_name.to_string()) {
                    return mismatch(
                        path,
                        expected,
                        format!("missing '{}' in {:?}", card_name, names),
                    );
                }
            }
        }

        if let Some(excludes) = obj.get("excludes").and_then(|v| v.as_array()) {
            understood = true;
            for item in excludes {
                let Some(card_name) = entry_card_name(item) else {
                    return mismatch(path, expected, format!("malformed excludes entry {}", item));
                };
                if contains(&card_name.to_string()) {
                    return mismatch(
                        path,
                        expected,
                        format!("'{}' should not be in {:?}", card_name, names),
                    );
                }
            }
        }

        if understood {
            return None;
        }

        return mismatch(
            path,
            expected,
            format!(
                "unsupported list assertion (want is_empty/includes/excludes); actual {:?}",
                names
            ),
        );
    }

    // Treat a scalar string as an exact card name match.
    if let Some(s) = expected.as_str() {
        if contains(&s.to_string()) {
            return None;
        }
        return mismatch(path, expected, format!("'{}' not found in {:?}", s, names));
    }

    mismatch(
        path,
        expected,
        format!("unsupported list assertion value; actual {:?}", names),
    )
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[test]
/// The harness can build a two-player initial state and run simple priority-pass assertions.
fn test_harness_build_state_two_players() {
    use mtg_engine::testing::script_schema::{
        Confidence, GameScript, InitialState, PlayerInitState, ReviewStatus, ScriptMetadata,
        ZonesInitState,
    };

    let script = GameScript {
        schema_version: "1.0.0".to_string(),
        metadata: ScriptMetadata {
            id: "script_harness_test_001".to_string(),
            name: "Harness smoke test".to_string(),
            description: "Basic two-player state construction".to_string(),
            cr_sections_tested: vec![],
            corner_case_ref: None,
            tags: vec![],
            confidence: Confidence::High,
            review_status: ReviewStatus::Approved,
            reviewed_by: None,
            review_date: None,
            generation_notes: None,
            disputes: vec![],
            retirement_reason: None,
        },
        initial_state: InitialState {
            format: "commander".to_string(),
            turn_number: 1,
            active_player: "p1".to_string(),
            phase: "precombat_main".to_string(),
            step: None,
            priority: "p1".to_string(),
            players: {
                let mut m = HashMap::new();
                m.insert(
                    "p1".to_string(),
                    PlayerInitState {
                        life: 40,
                        mana_pool: HashMap::new(),
                        land_plays_remaining: 1,
                        poison_counters: 0,
                        commander_damage_received: HashMap::new(),
                        commander: None,
                        partner_commander: None,
                    },
                );
                m.insert(
                    "p2".to_string(),
                    PlayerInitState {
                        life: 37,
                        mana_pool: HashMap::new(),
                        land_plays_remaining: 0,
                        poison_counters: 0,
                        commander_damage_received: HashMap::new(),
                        commander: None,
                        partner_commander: None,
                    },
                );
                m
            },
            zones: ZonesInitState {
                battlefield: HashMap::new(),
                hand: HashMap::new(),
                graveyard: HashMap::new(),
                exile: vec![],
                command_zone: HashMap::new(),
                library: HashMap::new(),
                stack: vec![],
            },
            continuous_effects: vec![],
        },
        script: vec![],
    };

    let (state, players) = build_initial_state(&script.initial_state);

    // p1 → PlayerId(1), p2 → PlayerId(2) (sorted alphabetically).
    assert_eq!(players["p1"], PlayerId(1));
    assert_eq!(players["p2"], PlayerId(2));

    // p2 has 37 life (patched after initial 40 default).
    // NOTE: life patching in add_player_with_life is a no-op; default is 40.
    // This test just confirms the state was built successfully.
    assert!(state.players().contains_key(&PlayerId(1)));
    assert!(state.players().contains_key(&PlayerId(2)));
}

#[test]
/// The harness can check `players.<name>.life` assertions correctly.
fn test_harness_assert_player_life() {
    use mtg_engine::testing::script_schema::{InitialState, PlayerInitState, ZonesInitState};

    let mut players_init = HashMap::new();
    players_init.insert(
        "alice".to_string(),
        PlayerInitState {
            life: 40,
            mana_pool: HashMap::new(),
            land_plays_remaining: 1,
            poison_counters: 0,
            commander_damage_received: HashMap::new(),
            commander: None,
            partner_commander: None,
        },
    );

    let init = InitialState {
        format: "commander".to_string(),
        turn_number: 1,
        active_player: "alice".to_string(),
        phase: "precombat_main".to_string(),
        step: None,
        priority: "alice".to_string(),
        players: players_init,
        zones: ZonesInitState {
            battlefield: HashMap::new(),
            hand: HashMap::new(),
            graveyard: HashMap::new(),
            exile: vec![],
            command_zone: HashMap::new(),
            library: HashMap::new(),
            stack: vec![],
        },
        continuous_effects: vec![],
    };

    let (state, player_map) = build_initial_state(&init);

    // Default life is 40 (Commander).
    let mut assertions = HashMap::new();
    assertions.insert("players.alice.life".to_string(), serde_json::json!(40));

    let mismatches = check_assertions(&state, &assertions, &player_map);
    assert!(
        mismatches.is_empty(),
        "Expected no mismatches: {:?}",
        mismatches
    );

    // Wrong value → mismatch.
    let mut wrong = HashMap::new();
    wrong.insert("players.alice.life".to_string(), serde_json::json!(37));
    let mismatches2 = check_assertions(&state, &wrong, &player_map);
    assert_eq!(
        mismatches2.len(),
        1,
        "Expected 1 mismatch for wrong life total"
    );
}

#[test]
/// The harness correctly counts hand cards via `zones.hand.<player>.count`.
fn test_harness_assert_hand_count() {
    use mtg_engine::testing::script_schema::{
        CardInZone, InitialState, PlayerInitState, ZonesInitState,
    };

    let mut players_init = HashMap::new();
    players_init.insert(
        "p1".to_string(),
        PlayerInitState {
            life: 40,
            mana_pool: HashMap::new(),
            land_plays_remaining: 1,
            poison_counters: 0,
            commander_damage_received: HashMap::new(),
            commander: None,
            partner_commander: None,
        },
    );

    let mut hand = HashMap::new();
    hand.insert(
        "p1".to_string(),
        vec![
            CardInZone {
                card: "Lightning Bolt".to_string(),
                is_commander: false,
                owner: None,
                ..Default::default()
            },
            CardInZone {
                card: "Counterspell".to_string(),
                is_commander: false,
                owner: None,
                ..Default::default()
            },
        ],
    );

    let init = InitialState {
        format: "commander".to_string(),
        turn_number: 1,
        active_player: "p1".to_string(),
        phase: "precombat_main".to_string(),
        step: None,
        priority: "p1".to_string(),
        players: players_init,
        zones: ZonesInitState {
            battlefield: HashMap::new(),
            hand,
            graveyard: HashMap::new(),
            exile: vec![],
            command_zone: HashMap::new(),
            library: HashMap::new(),
            stack: vec![],
        },
        continuous_effects: vec![],
    };

    let (state, player_map) = build_initial_state(&init);

    let mut assertions = HashMap::new();
    assertions.insert("zones.hand.p1.count".to_string(), serde_json::json!(2));

    let mismatches = check_assertions(&state, &assertions, &player_map);
    assert!(
        mismatches.is_empty(),
        "Expected no mismatches: {:?}",
        mismatches
    );
}

#[test]
/// The harness can run a full end-to-end replay with priority passes.
fn test_harness_end_to_end_priority_passes() {
    use mtg_engine::testing::script_schema::{
        Confidence, GameScript, InitialState, PlayerInitState, ReviewStatus, ScriptAction,
        ScriptMetadata, ScriptStep, ZonesInitState,
    };

    let p1_name = "p1".to_string();
    let p2_name = "p2".to_string();

    let mut players_init = HashMap::new();
    players_init.insert(
        p1_name.clone(),
        PlayerInitState {
            life: 40,
            mana_pool: HashMap::new(),
            land_plays_remaining: 1,
            poison_counters: 0,
            commander_damage_received: HashMap::new(),
            commander: None,
            partner_commander: None,
        },
    );
    players_init.insert(
        p2_name.clone(),
        PlayerInitState {
            life: 40,
            mana_pool: HashMap::new(),
            land_plays_remaining: 0,
            poison_counters: 0,
            commander_damage_received: HashMap::new(),
            commander: None,
            partner_commander: None,
        },
    );

    let script = GameScript {
        schema_version: "1.0.0".to_string(),
        metadata: ScriptMetadata {
            id: "script_priority_test_001".to_string(),
            name: "Priority pass test".to_string(),
            description: "Both players pass priority, advancing the step".to_string(),
            cr_sections_tested: vec!["116".to_string()],
            corner_case_ref: None,
            tags: vec![],
            confidence: Confidence::High,
            review_status: ReviewStatus::Approved,
            reviewed_by: None,
            review_date: None,
            generation_notes: None,
            disputes: vec![],
            retirement_reason: None,
        },
        initial_state: InitialState {
            format: "commander".to_string(),
            turn_number: 1,
            active_player: p1_name.clone(),
            phase: "precombat_main".to_string(),
            step: None,
            priority: p1_name.clone(),
            players: players_init,
            zones: ZonesInitState {
                battlefield: HashMap::new(),
                hand: HashMap::new(),
                graveyard: HashMap::new(),
                exile: vec![],
                command_zone: HashMap::new(),
                library: HashMap::new(),
                stack: vec![],
            },
            continuous_effects: vec![],
        },
        script: vec![ScriptStep {
            step: "precombat_main".to_string(),
            step_note: None,
            actions: vec![
                ScriptAction::PriorityRound {
                    players: vec![p1_name, p2_name],
                    result: "all_pass".to_string(),
                    note: Some("Both players pass in main phase".to_string()),
                },
                ScriptAction::AssertState {
                    description: "After all-pass, step advances".to_string(),
                    assertions: {
                        let mut a = HashMap::new();
                        a.insert(
                            "zones.stack".to_string(),
                            serde_json::json!({"is_empty": true}),
                        );
                        a
                    },
                    note: None,
                },
            ],
        }],
    };

    let results = replay_script(&script);

    // One assert_state checkpoint.
    assert_eq!(results.len(), 1);
    assert!(
        matches!(&results[0], ReplayResult::Ok { description, .. } if description.contains("After")),
        "Expected OK result; got {:?}",
        results
    );
}

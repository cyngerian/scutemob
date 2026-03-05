//! Legal action enumeration for the game simulator.
//!
//! Defines the `LegalAction` enum (all possible player actions) and the
//! `LegalActionProvider` trait. `StubProvider` implements basic checks
//! without deep engine knowledge — enough to play games, but misses edge
//! cases that a full engine implementation would catch.

use mtg_engine::{
    AttackTarget, CardType, GameState, KeywordAbility, ObjectId, PlayerId, Step, ZoneId,
};

/// A legal action a player may take at this moment.
#[derive(Clone, Debug)]
pub enum LegalAction {
    PassPriority,
    Concede,
    PlayLand {
        card: ObjectId,
    },
    CastSpell {
        card: ObjectId,
        from_zone: ZoneId,
    },
    TapForMana {
        source: ObjectId,
        ability_index: usize,
    },
    ActivateAbility {
        source: ObjectId,
        ability_index: usize,
    },
    DeclareAttackers {
        eligible: Vec<ObjectId>,
        targets: Vec<AttackTarget>,
    },
    DeclareBlockers {
        eligible: Vec<ObjectId>,
        attackers: Vec<ObjectId>,
    },
    TakeMulligan,
    KeepHand,
    ReturnCommanderToCommandZone {
        object_id: ObjectId,
    },
    LeaveCommanderInZone {
        object_id: ObjectId,
    },
}

/// Trait for enumerating legal actions from a game state.
pub trait LegalActionProvider: Send + Sync {
    fn legal_actions(&self, state: &GameState, player: PlayerId) -> Vec<LegalAction>;
}

/// Basic legal action enumeration — enough to play games, but misses
/// edge cases (flashback, escape, foretell, activated abilities on
/// permanents, etc.) that the full engine implementation will catch.
///
/// **B6 update (2026-03-05)**: Batch 6 alt-cost keywords (Bargain, Emerge, Spectacle,
/// Surge, Casualty, Assist) are fully implemented in the engine. The random_bot passes
/// `alt_cost: None` and `bargain_sacrifice/emerge_sacrifice/etc: None` — it never
/// attempts alt-cost casts. Full behavioral support (bot deciding to use alt costs based
/// on game state) is a W2 TUI task; see `docs/workstream-coordination.md` Phase 2.
pub struct StubProvider;

impl LegalActionProvider for StubProvider {
    fn legal_actions(&self, state: &GameState, player: PlayerId) -> Vec<LegalAction> {
        let mut actions = Vec::new();

        // Handle pending commander zone choices first
        if let Some((_pending_player, obj_id)) = state
            .pending_commander_zone_choices
            .iter()
            .find(|(p, _)| *p == player)
        {
            actions.push(LegalAction::ReturnCommanderToCommandZone { object_id: *obj_id });
            actions.push(LegalAction::LeaveCommanderInZone { object_id: *obj_id });
            return actions;
        }

        // Mulligan phase
        if state.turn.is_first_turn_of_game && state.turn.turn_number == 0 {
            actions.push(LegalAction::TakeMulligan);
            actions.push(LegalAction::KeepHand);
            return actions;
        }

        // Check if this player has priority
        if state.turn.priority_holder != Some(player) {
            return actions;
        }

        // Always available: pass priority
        // (Concede is intentionally omitted — bots should never auto-concede.
        // The human player can still quit via 'q'.)
        actions.push(LegalAction::PassPriority);

        let is_main_phase = matches!(state.turn.step, Step::PreCombatMain | Step::PostCombatMain);
        let stack_empty = state.stack_objects.is_empty();
        let is_active = state.turn.active_player == player;

        // Play lands: hand lands, main phase, stack empty, active player,
        // land plays remaining
        if is_main_phase && stack_empty && is_active {
            if let Ok(p) = state.player(player) {
                if p.land_plays_remaining > 0 {
                    let hand = ZoneId::Hand(player);
                    for obj in state.objects_in_zone(&hand) {
                        if obj.characteristics.card_types.contains(&CardType::Land) {
                            actions.push(LegalAction::PlayLand { card: obj.id });
                        }
                    }
                }
            }
        }

        // Cast spells from hand
        {
            let hand = ZoneId::Hand(player);
            for obj in state.objects_in_zone(&hand) {
                let is_land = obj.characteristics.card_types.contains(&CardType::Land);
                if is_land {
                    continue;
                }

                let is_instant = obj.characteristics.card_types.contains(&CardType::Instant);
                let has_flash = obj
                    .characteristics
                    .keywords
                    .contains(&KeywordAbility::Flash);

                // Timing check: instants and flash anytime with priority;
                // sorcery-speed only main phase + stack empty + active player
                let can_cast = if is_instant || has_flash {
                    true
                } else {
                    is_main_phase && stack_empty && is_active
                };

                if can_cast {
                    // Basic mana affordability check
                    if let Some(ref cost) = obj.characteristics.mana_cost {
                        if can_afford(state, player, cost) {
                            actions.push(LegalAction::CastSpell {
                                card: obj.id,
                                from_zone: hand,
                            });
                        }
                    }
                }
            }
        }

        // Tap for mana: untapped permanents with mana abilities on battlefield
        for obj in state.objects_in_zone(&ZoneId::Battlefield) {
            if obj.controller != player {
                continue;
            }
            if obj.status.tapped {
                continue;
            }
            for (idx, ability) in obj.characteristics.mana_abilities.iter().enumerate() {
                if ability.requires_tap {
                    actions.push(LegalAction::TapForMana {
                        source: obj.id,
                        ability_index: idx,
                    });
                }
            }
        }

        // Declare attackers: untapped creatures without summoning sickness
        // (unless haste) during DeclareAttackers step when active player
        if state.turn.step == Step::DeclareAttackers && is_active && stack_empty {
            let mut eligible = Vec::new();
            let mut targets = Vec::new();

            for obj in state.objects_in_zone(&ZoneId::Battlefield) {
                if obj.controller != player {
                    continue;
                }
                if !obj.characteristics.card_types.contains(&CardType::Creature) {
                    continue;
                }
                if obj.status.tapped {
                    continue;
                }
                if obj
                    .characteristics
                    .keywords
                    .contains(&KeywordAbility::Defender)
                {
                    continue;
                }
                if obj.has_summoning_sickness
                    && !obj
                        .characteristics
                        .keywords
                        .contains(&KeywordAbility::Haste)
                {
                    continue;
                }
                eligible.push(obj.id);
            }

            // Valid attack targets: opponents
            for p in state.active_players() {
                if p != player {
                    targets.push(AttackTarget::Player(p));
                }
            }

            if !eligible.is_empty() && !targets.is_empty() {
                actions.push(LegalAction::DeclareAttackers { eligible, targets });
            }
        }

        // Declare blockers: untapped creatures during DeclareBlockers step
        if state.turn.step == Step::DeclareBlockers && stack_empty {
            if let Some(ref combat) = state.combat {
                if !combat.attackers.is_empty() {
                    let mut eligible = Vec::new();
                    let mut attacker_ids: Vec<ObjectId> = Vec::new();

                    // Defending player(s) can block
                    for obj in state.objects_in_zone(&ZoneId::Battlefield) {
                        if obj.controller != player {
                            continue;
                        }
                        if !obj.characteristics.card_types.contains(&CardType::Creature) {
                            continue;
                        }
                        if obj.status.tapped {
                            continue;
                        }
                        eligible.push(obj.id);
                    }

                    for (attacker_id, _) in &combat.attackers {
                        attacker_ids.push(*attacker_id);
                    }

                    if !eligible.is_empty() && !attacker_ids.is_empty() {
                        actions.push(LegalAction::DeclareBlockers {
                            eligible,
                            attackers: attacker_ids,
                        });
                    }
                }
            }
        }

        // Activate non-mana abilities on battlefield permanents
        for obj in state.objects_in_zone(&ZoneId::Battlefield) {
            if obj.controller != player {
                continue;
            }
            for (idx, ability) in obj.characteristics.activated_abilities.iter().enumerate() {
                // Check tap requirement
                if ability.cost.requires_tap && obj.status.tapped {
                    continue;
                }
                // Sorcery-speed abilities
                if ability.sorcery_speed && !(is_main_phase && stack_empty && is_active) {
                    continue;
                }
                // Basic mana check
                if let Some(ref cost) = ability.cost.mana_cost {
                    if !can_afford(state, player, cost) {
                        continue;
                    }
                }
                actions.push(LegalAction::ActivateAbility {
                    source: obj.id,
                    ability_index: idx,
                });
            }
        }

        actions
    }
}

/// Mana affordability check: considers both mana pool and untapped sources.
/// Uses the mana solver for precise color-aware checking.
fn can_afford(state: &GameState, player: PlayerId, cost: &mtg_engine::ManaCost) -> bool {
    if state.player(player).is_err() {
        return false;
    }

    // If pool already has enough, no tapping needed
    if let Ok(p) = state.player(player) {
        let pool = &p.mana_pool;
        if pool.white >= cost.white
            && pool.blue >= cost.blue
            && pool.black >= cost.black
            && pool.red >= cost.red
            && pool.green >= cost.green
            && pool.colorless >= cost.colorless
            && pool.total() >= cost.mana_value()
        {
            return true;
        }
    }

    // Otherwise, check if mana solver can find a payment plan from untapped sources
    crate::mana_solver::solve_mana_payment(state, player, cost).is_some()
}

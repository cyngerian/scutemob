//! Legal action enumeration for the game simulator.
//!
//! Defines the `LegalAction` enum (all possible player actions) and the
//! `LegalActionProvider` trait. `StubProvider` implements basic checks
//! without deep engine knowledge — enough to play games, but misses edge
//! cases that a full engine implementation would catch.

use mtg_engine::{
    AbilityDefinition, AttackTarget, CardType, GameState, KeywordAbility, ObjectId, PlayerId, Step,
    ZoneId,
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
    /// CR 207.2c / CR 602: Bloodrush -- activated ability from hand targeting an attacking
    /// creature. Discards the card as cost, grants a P/T boost to the target until end of turn.
    ActivateBloodrush {
        /// The card with bloodrush in the player's hand.
        card: ObjectId,
        /// An attacking creature to target.
        target: ObjectId,
    },
    /// CR 702.171: Saddle a Mount by tapping creatures with total power >= N.
    /// Sorcery-speed only (active player, main phase, empty stack).
    SaddleMount {
        /// The Mount permanent to saddle.
        mount: ObjectId,
        /// Creatures to tap as the saddle cost (total power >= N).
        saddle_creatures: Vec<ObjectId>,
    },
    /// CR 702.140: Cast a card with Mutate using its mutate alternative cost,
    /// merging it with a target non-Human creature the caster owns.
    CastWithMutate {
        /// The card with Mutate in the player's hand.
        card: ObjectId,
        /// A non-Human creature the caster owns on the battlefield.
        mutate_target: ObjectId,
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
///
/// **B10 update (2026-03-07)**: Batch 10 ETB/dies pattern keywords (Devour, Backup,
/// Champion, Umbra Armor, Living Metal, Soulbond, Fortify) are fully implemented in the
/// engine. StubProvider handles these automatically via engine resolution — no LegalAction
/// changes needed. Soulbond pairing choice and Champion target choice are auto-selected
/// by the engine during trigger resolution. Fortify activation is handled by
/// `LegalAction::ActivateAbility` (already emitted). No bot behavioral changes needed.
///
/// **B12–B14 + Mutate update (2026-03-08)**:
/// - Bloodrush (B12): `LegalAction::ActivateBloodrush` emitted when a hand card has
///   `AbilityDefinition::Bloodrush` and there is at least one attacking creature.
/// - Saddle (B13): `LegalAction::SaddleMount` emitted when a Mount is on the battlefield
///   and the player controls untapped creatures with total power >= the Saddle N value.
///   Sorcery-speed only. StubProvider picks the first valid greedy set.
/// - Mutate: `LegalAction::CastWithMutate` emitted when a card in hand has
///   `KeywordAbility::Mutate` (and `AbilityDefinition::MutateCost`) and the player
///   owns a non-Human creature on the battlefield. Mutate is an alternative cost —
///   random_bot casts with `alt_cost: Some(AltCostKind::Mutate)` and `mutate_on_top: true`.
/// - Enrage/Alliance (B12), Collect Evidence (B13), Blood tokens/Reconfigure (B14):
///   all passive or handled via existing `ActivateAbility`/`CastSpell` paths — no new
///   `LegalAction` variants needed.
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

        // ── Bloodrush (CR 207.2c / B12) ─────────────────────────────────────────
        // Bloodrush is an activated ability from hand: discard the card to grant
        // a P/T boost to an attacking creature. Legal any time the player has
        // priority and an attacking creature exists (instant speed, no stack restriction).
        {
            // Collect attacking creature IDs once.
            let attacking: Vec<ObjectId> = if let Some(ref combat) = state.combat {
                combat.attackers.keys().copied().collect()
            } else {
                Vec::new()
            };

            if !attacking.is_empty() {
                let hand = ZoneId::Hand(player);
                for obj in state.objects_in_zone(&hand) {
                    // Check card_id is set and card definition has AbilityDefinition::Bloodrush.
                    let has_bloodrush = obj
                        .card_id
                        .as_ref()
                        .and_then(|cid| state.card_registry.get(cid.clone()))
                        .map(|def| {
                            def.abilities
                                .iter()
                                .any(|a| matches!(a, AbilityDefinition::Bloodrush { .. }))
                        })
                        .unwrap_or(false);

                    if !has_bloodrush {
                        continue;
                    }

                    // Check mana affordability for the bloodrush cost.
                    let bloodrush_cost = obj
                        .card_id
                        .as_ref()
                        .and_then(|cid| state.card_registry.get(cid.clone()))
                        .and_then(|def| {
                            def.abilities.iter().find_map(|a| {
                                if let AbilityDefinition::Bloodrush { cost, .. } = a {
                                    Some(cost.clone())
                                } else {
                                    None
                                }
                            })
                        });

                    if let Some(cost) = bloodrush_cost {
                        if can_afford(state, player, &cost) {
                            // Emit one action per attacking creature target.
                            for &target in &attacking {
                                actions.push(LegalAction::ActivateBloodrush {
                                    card: obj.id,
                                    target,
                                });
                            }
                        }
                    }
                }
            }
        }

        // ── Saddle (CR 702.171 / B13) ────────────────────────────────────────────
        // Sorcery-speed (active player, main phase, empty stack). The player taps
        // untapped creatures they control (excluding the Mount itself) with total
        // power >= N to saddle the Mount.
        if is_main_phase && stack_empty && is_active {
            // Collect untapped creatures the player controls (potential saddle creatures).
            let untapped_creatures: Vec<(ObjectId, i32)> = state
                .objects_in_zone(&ZoneId::Battlefield)
                .into_iter()
                .filter(|o| {
                    o.controller == player
                        && !o.status.tapped
                        && o.characteristics.card_types.contains(&CardType::Creature)
                })
                .map(|o| (o.id, o.characteristics.power.unwrap_or(0)))
                .collect();

            // Find Mounts with Saddle(N).
            for obj in state.objects_in_zone(&ZoneId::Battlefield) {
                if obj.controller != player {
                    continue;
                }
                let saddle_n = obj.characteristics.keywords.iter().find_map(|kw| {
                    if let KeywordAbility::Saddle(n) = kw {
                        Some(*n)
                    } else {
                        None
                    }
                });
                let saddle_n = match saddle_n {
                    Some(n) => n,
                    None => continue,
                };

                // Greedy selection: pick untapped creatures (excluding the Mount itself)
                // until we meet or exceed the power threshold.
                let mut chosen: Vec<ObjectId> = Vec::new();
                let mut total_power: i32 = 0;
                for &(cid, power) in &untapped_creatures {
                    if cid == obj.id {
                        continue; // Can't use the Mount itself as a saddle creature.
                    }
                    chosen.push(cid);
                    total_power += power;
                    if total_power >= saddle_n as i32 {
                        break;
                    }
                }

                if total_power >= saddle_n as i32 {
                    actions.push(LegalAction::SaddleMount {
                        mount: obj.id,
                        saddle_creatures: chosen,
                    });
                }
            }
        }

        // ── Mutate (CR 702.140) ──────────────────────────────────────────────────
        // Mutate is an alternative cost. Cards with Mutate in hand may be cast merging
        // with a non-Human creature the caster OWNS on the battlefield (CR 702.140a).
        // Timing follows normal spell timing (instant if the card is an instant / has flash;
        // otherwise sorcery-speed). StubProvider conservatively emits at sorcery-speed only
        // for creature spells (the common case — almost all mutate cards are creatures).
        if is_main_phase && stack_empty && is_active {
            // Collect non-Human creatures the player OWNS on the battlefield.
            let non_human_own: Vec<ObjectId> = state
                .objects_in_zone(&ZoneId::Battlefield)
                .into_iter()
                .filter(|o| {
                    // Owner check (not controller — CR 702.140a says "you own").
                    o.owner == player
                        && o.characteristics.card_types.contains(&CardType::Creature)
                        && !o
                            .characteristics
                            .subtypes
                            .contains(&mtg_engine::SubType("Human".to_string()))
                })
                .map(|o| o.id)
                .collect();

            if !non_human_own.is_empty() {
                let hand = ZoneId::Hand(player);
                for obj in state.objects_in_zone(&hand) {
                    if !obj
                        .characteristics
                        .keywords
                        .contains(&KeywordAbility::Mutate)
                    {
                        continue;
                    }

                    // Look up the mutate cost from the card registry.
                    let mutate_cost = obj
                        .card_id
                        .as_ref()
                        .and_then(|cid| state.card_registry.get(cid.clone()))
                        .and_then(|def| {
                            def.abilities.iter().find_map(|a| {
                                if let AbilityDefinition::MutateCost { cost } = a {
                                    Some(cost.clone())
                                } else {
                                    None
                                }
                            })
                        });

                    let mutate_cost = match mutate_cost {
                        Some(c) => c,
                        None => continue, // No MutateCost defined — skip.
                    };

                    if !can_afford(state, player, &mutate_cost) {
                        continue;
                    }

                    // Emit one action per valid mutate target.
                    for &target in &non_human_own {
                        actions.push(LegalAction::CastWithMutate {
                            card: obj.id,
                            mutate_target: target,
                        });
                    }
                }
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

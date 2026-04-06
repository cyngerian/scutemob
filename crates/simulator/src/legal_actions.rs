//! Legal action enumeration for the game simulator.
//!
//! Defines the `LegalAction` enum (all possible player actions) and the
//! `LegalActionProvider` trait. `StubProvider` implements basic checks
//! without deep engine knowledge — enough to play games, but misses edge
//! cases that a full engine implementation would catch.

use mtg_engine::{
    AbilityDefinition, AttackTarget, CardType, CounterType, FaceDownKind, FlashGrantFilter,
    GameRestriction, GameState, KeywordAbility, ManaCost, ObjectId, PlayerId, Step,
    TurnFaceUpMethod, ZoneId,
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
    /// CR 702.37e / CR 702.168b / CR 701.40b / CR 701.58b: Turn a face-down permanent
    /// face up. This is a special action (no stack, no priority needed beyond having it).
    /// Valid at any time the player has priority.
    TurnFaceUp {
        /// The face-down permanent to turn face up.
        permanent: ObjectId,
        /// The method to use for paying the face-up cost.
        method: TurnFaceUpMethod,
    },
    /// CR 606: Activate a loyalty ability on a planeswalker.
    /// Sorcery-speed, empty stack, once per permanent per turn.
    ActivateLoyaltyAbility {
        /// The planeswalker permanent.
        source: ObjectId,
        /// Which loyalty ability (filtered index).
        ability_index: usize,
    },
    /// CR 702.37a / CR 702.37b / CR 702.168b: Cast a card with Morph/Megamorph/Disguise
    /// face-down for {3} (or disguise cost) as a 2/2 creature with no name/text/subtypes.
    CastMorphFaceDown {
        /// The card in hand to cast face-down.
        card: ObjectId,
        /// The alt-cost kind to use (always AltCostKind::Morph).
        face_down_kind: FaceDownKind,
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
///
/// **PB-22 S7 (2026-03-21) — Adventure gap (deferred to W2)**:
/// TODO(W2): Adventure casting paths are not offered to the bot. Two gaps:
///   (a) `CastAsAdventure { card: ObjectId }` — cast a card in hand as its Adventure half
///       (CR 715.3); requires checking `adventure_face.is_some()` on CardDefinition and
///       comparing mana against the adventure_face cost. The engine supports
///       `alt_cost: Some(AltCostKind::Adventure)` on CastSpell; bot never sets it.
///   (b) `CastFromAdventureExile { card: ObjectId }` — cast a creature from adventure exile
///       (CR 715.3d); requires checking `adventure_exiled_by == Some(player)` on GameObject.
///   Both gaps are consistent with other alt-cost keywords (Spectacle, Surge, etc.) where
///   the bot always uses `alt_cost: None`. Deferred to W2 TUI/simulator improvements.
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

        // PB-18: Pre-compute restriction flags for this player.
        let cast_restricted = is_cast_restricted_by_stax(state, player);

        // Cast spells from hand
        if !cast_restricted {
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

                // CR 601.3b: Check if player has an active flash grant for this spell.
                let has_flash_grant = state.flash_grants.iter().any(|g| {
                    if g.player != player {
                        return false;
                    }
                    match &g.filter {
                        FlashGrantFilter::AllSpells => true,
                        FlashGrantFilter::Sorceries => {
                            obj.characteristics.card_types.contains(&CardType::Sorcery)
                        }
                        FlashGrantFilter::GreenCreatures => {
                            obj.characteristics.card_types.contains(&CardType::Creature)
                                && obj
                                    .characteristics
                                    .colors
                                    .contains(&mtg_engine::Color::Green)
                        }
                    }
                });
                // Timing check: instants and flash anytime with priority;
                // sorcery-speed only main phase + stack empty + active player
                let can_cast = if is_instant || has_flash || has_flash_grant {
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
                // PB-18 review Finding 4: filter abilities blocked by active restrictions.
                // Mirrors check_activate_restrictions in rules/abilities.rs.
                if is_ability_restricted_by_stax(state, player, obj.id) {
                    continue;
                }
                actions.push(LegalAction::ActivateAbility {
                    source: obj.id,
                    ability_index: idx,
                });
            }
        }

        // ── Loyalty abilities (CR 606) ────────────────────────────────────────────
        // Sorcery-speed, stack empty, once per permanent per turn.
        if is_main_phase && stack_empty && is_active {
            for obj in state.objects_in_zone(&ZoneId::Battlefield) {
                if obj.controller != player {
                    continue;
                }
                if obj.loyalty_ability_activated_this_turn {
                    continue;
                }
                if !obj
                    .characteristics
                    .card_types
                    .contains(&CardType::Planeswalker)
                {
                    continue;
                }
                // Look up card definition for loyalty abilities.
                if let Some(ref cid) = obj.card_id {
                    if let Some(def) = state.card_registry.get(cid.clone()) {
                        let loyalty_count = obj
                            .counters
                            .get(&CounterType::Loyalty)
                            .copied()
                            .unwrap_or(0);
                        for (idx, ability) in def.abilities.iter().enumerate() {
                            if let mtg_engine::AbilityDefinition::LoyaltyAbility { cost, .. } =
                                ability
                            {
                                // CR 606.6: check sufficient loyalty for negative costs.
                                let can_afford_loyalty = match cost {
                                    mtg_engine::LoyaltyCost::Plus(_)
                                    | mtg_engine::LoyaltyCost::Zero => true,
                                    mtg_engine::LoyaltyCost::Minus(n) => loyalty_count >= *n,
                                    mtg_engine::LoyaltyCost::MinusX => true, // X can be 0
                                };
                                if can_afford_loyalty {
                                    // Use filtered index: count LoyaltyAbility entries up to idx.
                                    let filtered_idx = def.abilities[..=idx]
                                        .iter()
                                        .filter(|a| {
                                            matches!(
                                                a,
                                                mtg_engine::AbilityDefinition::LoyaltyAbility { .. }
                                            )
                                        })
                                        .count()
                                        - 1;
                                    actions.push(LegalAction::ActivateLoyaltyAbility {
                                        source: obj.id,
                                        ability_index: filtered_idx,
                                    });
                                }
                            }
                        }
                    }
                }
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

        // ── TurnFaceUp (CR 702.37e) ──────────────────────────────────────────────
        // Special action: turn a face-down permanent face up at any time the player
        // has priority (no sorcery restriction — CR 116.2b). The player must control
        // the permanent and be able to pay the turn-face-up cost.
        for obj in state.objects_in_zone(&ZoneId::Battlefield) {
            if obj.controller != player {
                continue;
            }
            if !obj.status.face_down {
                continue;
            }
            let face_down_kind = match &obj.face_down_as {
                Some(k) => k.clone(),
                None => continue,
            };

            let card_def = obj
                .card_id
                .as_ref()
                .and_then(|cid| state.card_registry.get(cid.clone()));

            match face_down_kind {
                FaceDownKind::Morph | FaceDownKind::Megamorph => {
                    // Check for Morph or Megamorph ability in the card definition.
                    if let Some(def) = &card_def {
                        for ability in &def.abilities {
                            match ability {
                                AbilityDefinition::Morph { cost } => {
                                    if can_afford(state, player, cost) {
                                        actions.push(LegalAction::TurnFaceUp {
                                            permanent: obj.id,
                                            method: TurnFaceUpMethod::MorphCost,
                                        });
                                    }
                                    break;
                                }
                                AbilityDefinition::Megamorph { cost } => {
                                    if can_afford(state, player, cost) {
                                        actions.push(LegalAction::TurnFaceUp {
                                            permanent: obj.id,
                                            method: TurnFaceUpMethod::MorphCost,
                                        });
                                    }
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                FaceDownKind::Disguise => {
                    // Disguise turn-face-up cost is in the Disguise ability.
                    if let Some(def) = &card_def {
                        for ability in &def.abilities {
                            if let AbilityDefinition::Disguise { cost } = ability {
                                if can_afford(state, player, cost) {
                                    actions.push(LegalAction::TurnFaceUp {
                                        permanent: obj.id,
                                        method: TurnFaceUpMethod::DisguiseCost,
                                    });
                                }
                                break;
                            }
                        }
                    }
                }
                FaceDownKind::Manifest | FaceDownKind::Cloak => {
                    // Manifest/Cloak: turn face up by paying printed mana cost, but only
                    // if the card is a creature card (CR 701.40b, CR 701.58b).
                    let is_creature = obj.characteristics.card_types.contains(&CardType::Creature);
                    // For Manifest/Cloak the raw card_types reflect the real card — check
                    // the card registry for the actual type line.
                    let def_is_creature = card_def
                        .as_ref()
                        .map(|def| def.types.card_types.contains(&CardType::Creature))
                        .unwrap_or(false);

                    if is_creature || def_is_creature {
                        let mana_cost = card_def
                            .as_ref()
                            .and_then(|def| def.mana_cost.clone())
                            .or_else(|| obj.characteristics.mana_cost.clone());
                        if let Some(cost) = mana_cost {
                            if can_afford(state, player, &cost) {
                                actions.push(LegalAction::TurnFaceUp {
                                    permanent: obj.id,
                                    method: TurnFaceUpMethod::ManaCost,
                                });
                            }
                        }
                    }

                    // Manifested/cloaked card with Morph/Megamorph can also use morph cost
                    // (CR 701.40c, CR 701.58c).
                    if let Some(def) = &card_def {
                        for ability in &def.abilities {
                            match ability {
                                AbilityDefinition::Morph { cost }
                                | AbilityDefinition::Megamorph { cost } => {
                                    if can_afford(state, player, cost) {
                                        actions.push(LegalAction::TurnFaceUp {
                                            permanent: obj.id,
                                            method: TurnFaceUpMethod::MorphCost,
                                        });
                                    }
                                    break;
                                }
                                AbilityDefinition::Disguise { cost } => {
                                    if can_afford(state, player, cost) {
                                        actions.push(LegalAction::TurnFaceUp {
                                            permanent: obj.id,
                                            method: TurnFaceUpMethod::DisguiseCost,
                                        });
                                    }
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // ── Morph/Megamorph/Disguise face-down cast (CR 702.37a / CR 702.168b) ──
        // Sorcery-speed only: active player, main phase, empty stack. The player may
        // cast a card with Morph/Megamorph/Disguise face-down for {3} (morph) or the
        // disguise cost. StubProvider conservatively emits sorcery-speed only.
        if is_main_phase && stack_empty && is_active {
            let morph_base_cost = ManaCost {
                generic: 3,
                ..Default::default()
            };
            let hand = ZoneId::Hand(player);
            for obj in state.objects_in_zone(&hand) {
                let is_land = obj.characteristics.card_types.contains(&CardType::Land);
                if is_land {
                    continue;
                }

                let card_def = obj
                    .card_id
                    .as_ref()
                    .and_then(|cid| state.card_registry.get(cid.clone()));

                let has_morph = card_def.as_ref().map(|def| {
                    def.abilities.iter().any(|a| {
                        matches!(
                            a,
                            AbilityDefinition::Morph { .. }
                                | AbilityDefinition::Megamorph { .. }
                                | AbilityDefinition::Disguise { .. }
                        )
                    })
                });

                if has_morph != Some(true) {
                    continue;
                }

                // Morph/Megamorph cast face-down costs {3}.
                let has_morph_or_mega = card_def.as_ref().map(|def| {
                    def.abilities.iter().any(|a| {
                        matches!(
                            a,
                            AbilityDefinition::Morph { .. } | AbilityDefinition::Megamorph { .. }
                        )
                    })
                });
                if has_morph_or_mega == Some(true) && can_afford(state, player, &morph_base_cost) {
                    actions.push(LegalAction::CastMorphFaceDown {
                        card: obj.id,
                        face_down_kind: FaceDownKind::Morph,
                    });
                }

                // Disguise cast costs its disguise cost.
                if let Some(def) = &card_def {
                    for ability in &def.abilities {
                        if let AbilityDefinition::Disguise { cost } = ability {
                            if can_afford(state, player, cost) {
                                actions.push(LegalAction::CastMorphFaceDown {
                                    card: obj.id,
                                    face_down_kind: FaceDownKind::Disguise,
                                });
                            }
                            break;
                        }
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

/// PB-18 review Finding 4: Check whether any active restriction prevents this player
/// from activating an ability of a specific source object.
///
/// Mirrors check_activate_restrictions in rules/abilities.rs. Only objects on the
/// battlefield are affected (zone-scope fix from Finding 3).
fn is_ability_restricted_by_stax(state: &GameState, player: PlayerId, source: ObjectId) -> bool {
    let active_player = state.turn.active_player;

    // Source must be on the battlefield for restrictions to apply (Finding 3).
    let source_on_battlefield = state
        .objects
        .get(&source)
        .map(|o| o.zone == ZoneId::Battlefield)
        .unwrap_or(false);

    if !source_on_battlefield {
        return false;
    }

    // Compute source card types once.
    let source_is_artifact = mtg_engine::rules::layers::calculate_characteristics(state, source)
        .map(|c| c.card_types.contains(&CardType::Artifact))
        .unwrap_or(false);

    let source_is_restricted_type =
        mtg_engine::rules::layers::calculate_characteristics(state, source)
            .map(|c| {
                c.card_types.contains(&CardType::Artifact)
                    || c.card_types.contains(&CardType::Creature)
                    || c.card_types.contains(&CardType::Enchantment)
            })
            .unwrap_or(false);

    for restriction in state.restrictions.iter() {
        let restriction_source_on_bf = state
            .objects
            .get(&restriction.source)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false);
        if !restriction_source_on_bf {
            continue;
        }

        let controller = restriction.controller;

        match &restriction.restriction {
            GameRestriction::ArtifactAbilitiesCantBeActivated => {
                if source_is_artifact {
                    return true;
                }
            }
            GameRestriction::OpponentsCantCastOrActivateDuringYourTurn => {
                if active_player == controller && player != controller && source_is_restricted_type
                {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

/// PB-18: Check whether any active restriction prevents this player from casting
/// any spell at all (MaxSpellsPerTurn, OpponentsCantCast*).
///
/// Returns true if the player is completely restricted from casting.
/// Does NOT check per-card restrictions (like Drannith Magistrate's zone restriction)
/// — those need per-card checking at a deeper level.
fn is_cast_restricted_by_stax(state: &GameState, player: PlayerId) -> bool {
    use mtg_engine::GameRestriction;

    let active_player = state.turn.active_player;

    for restriction in state.restrictions.iter() {
        // Skip restrictions whose source is no longer on the battlefield.
        let source_on_bf = state
            .objects
            .get(&restriction.source)
            .map(|o| matches!(o.zone, mtg_engine::ZoneId::Battlefield))
            .unwrap_or(false);
        if !source_on_bf {
            continue;
        }

        let controller = restriction.controller;

        match &restriction.restriction {
            GameRestriction::MaxSpellsPerTurn { max } => {
                let spells_cast = state
                    .players
                    .get(&player)
                    .map(|ps| ps.spells_cast_this_turn)
                    .unwrap_or(0);
                if spells_cast >= *max {
                    return true;
                }
            }
            GameRestriction::OpponentsCantCastDuringYourTurn => {
                if active_player == controller && player != controller {
                    return true;
                }
            }
            GameRestriction::OpponentsCantCastOrActivateDuringYourTurn => {
                if active_player == controller && player != controller {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

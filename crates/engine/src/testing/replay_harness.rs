use crate::state::combat::AttackTarget;
use crate::state::types::{AdditionalCost, AltCostKind, FaceDownKind, TurnFaceUpMethod};
use crate::state::{ActivatedAbility, ActivationCost, CounterType, SacrificeFilter};
use crate::testing::script_schema::{
    ActionTarget, AttackerDeclaration, BlockerDeclaration, EnlistDeclaration, InitialState,
};
use crate::{
    all_cards, register_commander_zone_replacements, AbilityDefinition, CardDefinition,
    CardEffectTarget, CardId, CardRegistry, CardType, Color, Command, Cost, DeathTriggerFilter,
    Designations, ETBTriggerFilter, Effect, EffectAmount, GameState, GameStateBuilder,
    KeywordAbility, ManaAbility, ManaColor, ObjectSpec, PlayerId, Step, TargetController,
    TimingRestriction, TriggerCondition, TriggerEvent, TriggeredAbilityDef, ZoneId,
};
use im::OrdMap;
/// Replay harness helpers — extracted from `crates/engine/tests/script_replay.rs`
/// so that external tools (e.g. `tools/replay-viewer`) can reuse the same
/// `build_initial_state` logic without code duplication.
///
/// The test file retains `replay_script()`, `check_assertions()`,
/// `AssertionMismatch`, and `ReplayResult` since those types are test-specific.
///
/// # Public surface
/// - [`build_initial_state`] — converts a [`GameScript`] initial-state block into a live `GameState`
/// - [`parse_step`] — maps phase string → [`Step`]
/// - [`card_name_to_id`] — converts display name → kebab-case `CardId`
/// - [`enrich_spec_from_def`] — fills in types/costs/keywords from a `CardDefinition`
/// - [`parse_counter_type`] — maps counter string → [`CounterType`]
/// - [`translate_player_action`] — maps a script `PlayerAction` string → `Command`
use std::collections::HashMap;
// ── Public API ────────────────────────────────────────────────────────────────
/// Build a [`GameState`] from a [`GameScript`]'s initial state description.
///
/// Returns the state and a mapping from script player names → [`PlayerId`].
///
/// Player names are sorted alphabetically and assigned `PlayerId(1)`, `PlayerId(2)`, …
/// This is deterministic for a given set of player names.
pub fn build_initial_state(init: &InitialState) -> (GameState, HashMap<String, PlayerId>) {
    // Sort player names deterministically.
    let mut names: Vec<String> = init.players.keys().cloned().collect();
    names.sort();
    let player_map: HashMap<String, PlayerId> = names
        .iter()
        .enumerate()
        .map(|(i, name)| (name.clone(), PlayerId(i as u64 + 1)))
        .collect();
    let active = player_map
        .get(&init.active_player)
        .copied()
        .unwrap_or(PlayerId(1));
    let step = parse_step(&init.phase);
    // Load card definitions once for registry and for spec enrichment.
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    // Build registry (for spell effect execution during resolution).
    let registry = CardRegistry::new(cards); // returns Arc<CardRegistry>
    let mut builder = GameStateBuilder::new()
        .at_step(step)
        .active_player(active)
        .with_registry(registry);
    // Add players with their initial life / mana.
    for name in &names {
        let pid = player_map[name];
        builder = builder.add_player(pid);
    }
    // Helper closure: build a card spec enriched with definition characteristics.
    let make_spec = |owner: PlayerId, name: &str, zone: ZoneId| -> ObjectSpec {
        let base = ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name));
        enrich_spec_from_def(base, &defs)
    };
    // Add battlefield permanents (under each player's control).
    for (ctrl_name, permanents) in &init.zones.battlefield {
        if let Some(&ctrl) = player_map.get(ctrl_name) {
            for perm in permanents {
                let mut spec = make_spec(ctrl, &perm.card, ZoneId::Battlefield);
                if perm.tapped {
                    spec = spec.tapped();
                }
                for (ctype, count) in &perm.counters {
                    if let Some(ct) = parse_counter_type(ctype) {
                        spec = spec.with_counter(ct, *count);
                    }
                }
                if perm.damage_marked > 0 {
                    spec = spec.with_damage(perm.damage_marked);
                }
                builder = builder.object(spec);
            }
        }
    }
    // Add hand cards.
    for (owner_name, hand_cards) in &init.zones.hand {
        if let Some(&owner) = player_map.get(owner_name) {
            for card in hand_cards {
                let owner_pid = if let Some(o) = &card.owner {
                    player_map.get(o).copied().unwrap_or(owner)
                } else {
                    owner
                };
                builder = builder.object(make_spec(owner_pid, &card.card, ZoneId::Hand(owner)));
            }
        }
    }
    // Add graveyard cards.
    for (owner_name, gy_cards) in &init.zones.graveyard {
        if let Some(&owner) = player_map.get(owner_name) {
            for card in gy_cards {
                builder = builder.object(make_spec(owner, &card.card, ZoneId::Graveyard(owner)));
            }
        }
    }
    // Add exile cards (including suspended cards with time counters).
    for card in &init.zones.exile {
        let owner = card
            .owner
            .as_deref()
            .and_then(|n| player_map.get(n))
            .copied()
            .unwrap_or(PlayerId(1));
        let mut spec = make_spec(owner, &card.card, ZoneId::Exile);
        for (ctype, count) in &card.counters {
            if let Some(ct) = parse_counter_type(ctype) {
                spec = spec.with_counter(ct, *count);
            }
        }
        builder = builder.object(spec);
    }
    // Add library cards (top-to-bottom order).
    for (owner_name, lib_cards) in &init.zones.library {
        if let Some(&owner) = player_map.get(owner_name) {
            for card in lib_cards {
                builder = builder.object(make_spec(owner, &card.card, ZoneId::Library(owner)));
            }
        }
    }
    let mut state = builder.build().unwrap();
    // Patch life totals, mana pools, and land plays (can't do these via builder).
    for (name, pstate) in &init.players {
        if let Some(&pid) = player_map.get(name) {
            if let Some(ps) = state.players.get_mut(&pid) {
                ps.life_total = pstate.life;
                for (color_str, amount) in &pstate.mana_pool {
                    if let Some(color) = parse_mana_color(color_str) {
                        ps.mana_pool.add(color, *amount);
                    }
                }
                ps.land_plays_remaining = pstate.land_plays_remaining;
                ps.poison_counters = pstate.poison_counters;
            }
        }
    }
    // Patch is_suspended on exile objects (CR 702.62: cards suspended from hand have
    // this flag set so upkeep_actions can find them). The spec builder always sets it
    // false, so we must patch post-build using name matching in the exile zone.
    for card in &init.zones.exile {
        if card.is_suspended {
            let card_cid = card_name_to_id(&card.card);
            let target_id = state.objects.iter().find_map(|(&id, obj)| {
                if obj.zone == ZoneId::Exile
                    && obj.card_id.as_ref().map(|c| c.0.as_str()) == Some(card_cid.0.as_str())
                {
                    Some(id)
                } else {
                    None
                }
            });
            if let Some(id) = target_id {
                if let Some(obj) = state.objects.get_mut(&id) {
                    obj.designations.insert(Designations::SUSPENDED);
                }
            }
        }
    }
    // Register commanders: populate PlayerState::commander_ids from the script's
    // commander fields, then register zone-change replacement effects (CR 903.9).
    let mut any_commanders = false;
    for (name, pstate) in &init.players {
        if let Some(&pid) = player_map.get(name) {
            for cmdr in [&pstate.commander, &pstate.partner_commander]
                .into_iter()
                .flatten()
            {
                let cid = card_name_to_id(&cmdr.card);
                if let Some(ps) = state.players.get_mut(&pid) {
                    ps.commander_ids.push_back(cid);
                }
                any_commanders = true;
            }
        }
    }
    if any_commanders {
        register_commander_zone_replacements(&mut state);
    }
    (state, player_map)
}
/// Map a script `PlayerAction` string and its parameters to a [`Command`].
///
/// Returns `None` for unrecognized action strings (future-proof: new actions
/// are silently skipped rather than panicking).
#[allow(clippy::too_many_arguments)]
pub fn translate_player_action(
    action: &str,
    player: PlayerId,
    card_name: Option<&str>,
    ability_index: usize,
    targets: &[ActionTarget],
    attackers_decl: &[AttackerDeclaration],
    blockers_decl: &[BlockerDeclaration],
    convoke_names: &[String],
    improvise_names: &[String],
    delve_names: &[String],
    escape_names: &[String],
    kicked: bool,
    buyback: bool,
    enlist_decls: &[EnlistDeclaration],
    // CR 702.49a: For `activate_ninjutsu`, the name of the unblocked attacking
    // creature to return to its owner's hand. `None` for all other action types.
    attacker_name: Option<&str>,
    // CR 702.81a: For `cast_spell_retrace`, the name of the land card in the
    // player's hand to discard as the additional cost. `None` for all other action types.
    discard_land_name: Option<&str>,
    // CR 702.133a: For `cast_spell_jump_start`, the name of any card in the player's
    // hand to discard as the jump-start additional cost. `None` for all other action types.
    discard_card_name: Option<&str>,
    // CR 702.166a: For `cast_spell_bargain`, the name of an artifact, enchantment, or token
    // on the battlefield to sacrifice as the bargain additional cost. `None` for all other
    // action types or when the player chooses not to bargain.
    bargain_sacrifice_name: Option<&str>,
    // CR 702.119a: For `cast_spell_emerge`, the name of the creature on the battlefield to
    // sacrifice as the emerge alternative cost. Required for `cast_spell_emerge`; `None` for
    // all other action types.
    emerge_sacrifice_name: Option<&str>,
    // CR 702.153a: For `cast_spell_casualty`, the name of a creature on the battlefield to
    // sacrifice as the casualty additional cost. `None` for all other action types or when
    // the player chooses not to pay the casualty cost (casualty is optional).
    casualty_sacrifice_name: Option<&str>,
    // CR 702.132a: For `cast_spell_assist`, the name of the player who will pay part of
    // the generic mana cost. `None` for all other action types or when the caster
    // chooses not to use assist.
    assist_player_name: Option<&str>,
    // CR 702.132a: For `cast_spell_assist`, the amount of generic mana the assisting
    // player pays. Ignored when `assist_player_name` is `None`.
    assist_amount: u32,
    // CR 702.56a: For `cast_spell_replicate`, the number of times the replicate cost was paid.
    // 0 = not paid (no copies). N = paid N times → N copies created by trigger.
    // Ignored for all other action types.
    replicate_count: u32,
    // CR 702.47a: For `cast_spell_splice`, names of cards in the caster's hand to splice
    // onto the spell. Empty slice for all other action types.
    splice_card_names: &[String],
    // CR 702.120a: For `cast_spell_escalate`, the number of additional modes beyond the
    // first for which the escalate cost is paid. 0 for all other action types.
    escalate_modes: u32,
    // CR 700.2a / 601.2b: For `cast_spell_modal`, the explicit mode indices to choose.
    // Empty for all other action types or when mode[0] should be auto-selected.
    modes_chosen: Vec<usize>,
    // CR 702.97a: For `scavenge_card`, the name of the creature on the battlefield
    // to receive +1/+1 counters. Required for `scavenge_card`; `None` for all other
    // action types.
    target_creature_name: Option<&str>,
    // CR 107.3m: For `cast_spell` (and variants) with X in the mana cost.
    // The value chosen for X at cast time. 0 for non-X spells (default).
    x_value: u32,
    // CR 701.59a: For `cast_spell_collect_evidence`, names of cards in the caster's
    // graveyard to exile as the collect evidence additional cost.
    // Empty for all other action types or when the player chooses not to pay.
    collect_evidence_names: &[String],
    // CR 702.157a: For `cast_spell_squad`, the number of times the squad cost was paid
    // as an additional cost. 0 = not paid. N = paid N times -> N token copies on ETB.
    // Ignored for all other action types.
    squad_count: u32,
    // CR 702.140a: For `cast_spell_mutate`. True = spell goes on top of merged permanent.
    // False = spell goes underneath the existing target. Ignored for all other action types.
    mutate_on_top: bool,
    // CR 702.174a: For `cast_spell` with gift. The name of the opponent who receives the
    // gift benefit. `None` means the gift was not promised. Ignored for all other action types.
    gift_opponent_name: Option<&str>,
    // CR 602.2: For `activate_ability` with sacrifice-another-permanent cost. The name of
    // the permanent to sacrifice. `None` for abilities without sacrifice-other cost.
    sacrifice_card_name: Option<&str>,
    state: &GameState,
    players: &HashMap<String, PlayerId>,
) -> Option<Command> {
    match action {
        "pass_priority" => Some(Command::PassPriority { player }),
        "play_land" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            Some(Command::PlayLand {
                player,
                card: card_id,
            })
        }
        "cast_spell" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            // CR 702.51: Resolve each convoke creature name to an ObjectId on the battlefield.
            let convoke_ids: Vec<crate::state::game_object::ObjectId> = convoke_names
                .iter()
                .filter_map(|name| find_on_battlefield(state, player, name.as_str()))
                .collect();
            // CR 702.126: Resolve each improvise artifact name to an ObjectId on the battlefield.
            let improvise_ids: Vec<crate::state::game_object::ObjectId> = improvise_names
                .iter()
                .filter_map(|name| find_on_battlefield(state, player, name.as_str()))
                .collect();
            // CR 702.66: Resolve each delve card name to an ObjectId in the caster's graveyard.
            let delve_ids: Vec<crate::state::game_object::ObjectId> = delve_names
                .iter()
                .filter_map(|name| find_in_graveyard(state, player, name.as_str()))
                .collect();
            // CR 702.174a: Resolve gift opponent name to PlayerId if provided.
            let gift_pid = gift_opponent_name.and_then(|name| players.get(name).copied());
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: convoke_ids,
                improvise_artifacts: improvise_ids,
                delve_cards: delve_ids,
                kicker_times: if kicked { 1 } else { 0 },
                alt_cost: if buyback {
                    Some(AltCostKind::Buyback)
                } else {
                    None
                },
                prototype: false,
                modes_chosen: vec![],
                // CR 107.3m: Propagate x_value from the script action to CastSpell.
                x_value,
                face_down_kind: None,
                additional_costs: {
                    let mut costs = vec![];
                    if let Some(pid) = gift_pid {
                        costs.push(AdditionalCost::Gift { opponent: pid });
                    }
                    // CR 118.8: If the script specifies sacrifice_card for cast_spell,
                    // resolve it to an ObjectId and add AdditionalCost::Sacrifice.
                    if let Some(sac_name) = sacrifice_card_name {
                        if let Some(sac_id) = find_on_battlefield(state, player, sac_name) {
                            costs.push(AdditionalCost::Sacrifice(vec![sac_id]));
                        }
                    }
                    costs
                },
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.34a: Cast a spell with flashback from the player's graveyard.
        // The engine determines it's a flashback cast by checking the card's zone
        // (graveyard) and whether it has the Flashback keyword. No new Command variant
        // is needed — CastSpell handles flashback automatically when the source is
        // in the graveyard.
        "cast_spell_flashback" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.74a: Cast a spell with evoke from the player's hand.
        // The evoke cost (an alternative cost) is paid instead of the mana cost.
        "cast_spell_evoke" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Evoke),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.103a: Cast a spell with bestow from the player's hand.
        // The bestow cost (an alternative cost) is paid instead of the mana cost.
        "cast_spell_bestow" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Bestow),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.35a: Cast a madness card from exile by paying the madness cost.
        // The card is located in the caster's exile zone (put there by the discard
        // replacement effect). Madness is auto-detected from the card's zone + keyword.
        "cast_spell_madness" => {
            let card_id = find_in_exile(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.94a: Cast a miracle card from hand by paying the miracle cost.
        // The card is in hand (drawn this turn as first draw). A MiracleTrigger must
        // be on the stack (the player already chose to reveal via ChooseMiracle).
        "cast_spell_miracle" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Miracle),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.138a: Cast a spell with escape from the player's graveyard.
        // The escape cost (mana + exiling other cards) is paid instead of the mana cost.
        // The action uses the `escape` field: names of other cards to exile from graveyard.
        "cast_spell_escape" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            // Resolve escape exile card names to ObjectIds in the caster's graveyard.
            let exile_ids: Vec<crate::state::game_object::ObjectId> = escape_names
                .iter()
                .filter_map(|name| find_in_graveyard(state, player, name.as_str()))
                .collect();
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Escape),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![AdditionalCost::EscapeExile { cards: exile_ids }],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.94a: Choose to reveal a miracle card drawn this turn.
        // Sent in response to a `MiracleRevealChoiceRequired` event.
        // In scripts, `choose_miracle` always means `reveal: true` (to use miracle).
        // To decline, simply don't send this action — pass priority instead.
        "choose_miracle" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            Some(Command::ChooseMiracle {
                player,
                card: card_id,
                reveal: true,
            })
        }
        // CR 702.94a: Decline to reveal a miracle card drawn this turn.
        // Use this if the player drew a miracle card but does not want to cast it.
        "choose_miracle_decline" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            Some(Command::ChooseMiracle {
                player,
                card: card_id,
                reveal: false,
            })
        }
        "tap_for_mana" => {
            let source_id = find_on_battlefield(state, player, card_name?)?;
            // Assume ability index 0 for basic mana abilities.
            Some(Command::TapForMana {
                player,
                source: source_id,
                ability_index: 0,
            })
        }
        "activate_ability" => {
            let source_id = find_on_battlefield(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            // If the action specifies a discard_card_name (for Blood token activation),
            // resolve it to an ObjectId from the player's hand.
            let discard_card_id =
                discard_card_name.and_then(|name| find_in_hand(state, player, name));
            // If the script specifies sacrifice_card, resolve it to an ObjectId.
            let sacrifice_target_id =
                sacrifice_card_name.and_then(|name| find_on_battlefield(state, player, name));
            Some(Command::ActivateAbility {
                player,
                source: source_id,
                ability_index,
                targets: target_list,
                discard_card: discard_card_id,
                sacrifice_target: sacrifice_target_id,
                x_value: if x_value > 0 { Some(x_value) } else { None },
            })
        }
        // CR 606: Activate a loyalty ability on a planeswalker.
        "activate_loyalty_ability" => {
            let source_id = find_on_battlefield(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::ActivateLoyaltyAbility {
                player,
                source: source_id,
                ability_index,
                targets: target_list,
                x_value: if x_value > 0 { Some(x_value) } else { None },
            })
        }
        "cycle_card" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            Some(Command::CycleCard {
                player,
                card: card_id,
            })
        }
        // CR 207.2c: Activate a bloodrush ability from hand during combat.
        // card_name is the name of the bloodrush card in the player's hand.
        // targets[0] is the attacking creature to pump.
        "activate_bloodrush" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            let target_id = target_list
                .iter()
                .find_map(|t| match t {
                    crate::state::targeting::Target::Object(id) => Some(*id),
                    _ => None,
                })
                .ok_or_else(|| {
                    format!(
                        "activate_bloodrush: no valid Object target found for player {:?}",
                        player
                    )
                })
                .ok()?;
            Some(Command::ActivateBloodrush {
                player,
                card: card_id,
                target: target_id,
            })
        }
        // CR 702.57a: Activate a forecast ability from hand during the owner's upkeep.
        // card_name is the name of the forecast card in the player's hand.
        "activate_forecast" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::ActivateForecast {
                player,
                card: card_id,
                targets: target_list,
            })
        }
        // CR 702.84a: Activate an unearth ability from the graveyard.
        // card_name is the name of the card with unearth in the player's graveyard.
        "unearth_card" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            Some(Command::UnearthCard {
                player,
                card: card_id,
            })
        }
        // CR 702.128a: Activate an embalm ability from the graveyard.
        // card_name is the name of the card with embalm in the player's graveyard.
        // The card is exiled immediately as cost (CR 702.128a); token is created on resolution.
        "embalm_card" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            Some(Command::EmbalmCard {
                player,
                card: card_id,
            })
        }
        // CR 702.129a: Activate an eternalize ability from the graveyard.
        // card_name is the name of the card with eternalize in the player's graveyard.
        // The card is exiled immediately as cost (CR 702.129a); token is created on resolution.
        "eternalize_card" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            Some(Command::EternalizeCard {
                player,
                card: card_id,
            })
        }
        // CR 702.49a: Activate ninjutsu from hand (or command zone for commander
        // ninjutsu, CR 702.49d). `card_name` is the ninja card; `attacker_name` is
        // the unblocked attacking creature to return to its owner's hand.
        // JSON action shape: { "action_type": "activate_ninjutsu",
        //   "player": "p1", "card_name": "Ninja of the Deep Hours",
        //   "attacker_name": "Eager Construct", "mana_payment": { ... } }
        "activate_ninjutsu" => {
            let ninja_name = card_name?;
            let ninja_id = find_in_hand(state, player, ninja_name).or_else(|| {
                // Commander ninjutsu (CR 702.49d): also search the command zone.
                find_in_command_zone(state, player, ninja_name)
            })?;
            let atk_name = attacker_name?;
            let attacker_id = find_on_battlefield(state, player, atk_name)?;
            Some(Command::ActivateNinjutsu {
                player,
                ninja_card: ninja_id,
                attacker_to_return: attacker_id,
            })
        }
        // CR 702.141a: Activate an encore ability from the graveyard.
        // card_name is the name of the card with encore in the player's graveyard.
        // The card is exiled immediately as cost (CR 702.141a); tokens are created on resolution.
        "encore_card" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            Some(Command::EncoreCard {
                player,
                card: card_id,
            })
        }
        // CR 702.97a: Activate a scavenge ability from the graveyard.
        // card_name is the name of the card with scavenge in the player's graveyard.
        // target_creature_name is the name of the creature to receive +1/+1 counters.
        // The card is exiled immediately as cost (CR 702.97a); counters placed at resolution.
        "scavenge_card" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            let target_name = target_creature_name?;
            let target_id = find_on_battlefield_by_name(state, target_name)?;
            Some(Command::ScavengeCard {
                player,
                card: card_id,
                target_creature: target_id,
            })
        }
        // CR 702.62a: Suspend a card from the player's hand. card_name is the card
        // to suspend. The player pays the suspend cost; the card is exiled with N time
        // counters (as defined by the card's AbilityDefinition::Suspend).
        "suspend_card" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            Some(Command::SuspendCard {
                player,
                card: card_id,
            })
        }
        // CR 702.52a: Choose to use dredge instead of drawing. card_name is the
        // dredge card to return from graveyard; if absent, declines dredge (draws normally).
        "choose_dredge" => {
            let card = card_name.and_then(|name| find_in_graveyard(state, player, name));
            Some(Command::ChooseDredge { player, card })
        }
        // CR 702.59a: Pay the recover cost for a card in the graveyard. card_name is the
        // recover card to return; if absent, declines (exiles the recover card).
        "pay_recover" => {
            let (pay, recover_card) = if let Some(name) = card_name {
                let card_id = find_in_graveyard(state, player, name)?;
                (true, card_id)
            } else {
                let card_id = state
                    .pending_recover_payments
                    .iter()
                    .find(|(pid, _, _)| *pid == player)
                    .map(|(_, card_id, _)| *card_id)?;
                (false, card_id)
            };
            Some(Command::PayRecover {
                player,
                recover_card,
                pay,
            })
        }
        // CR 508.1: Declare attackers. Resolve creature names to ObjectIds on the
        // battlefield, and player names to AttackTarget::Player.
        "declare_attackers" => {
            let mut atk_pairs: Vec<(crate::state::ObjectId, AttackTarget)> = Vec::new();
            for decl in attackers_decl {
                let obj_id = find_on_battlefield(state, player, &decl.card)?;
                let target = if let Some(ref pname) = decl.target_player {
                    let &pid = players.get(pname.as_str())?;
                    AttackTarget::Player(pid)
                } else if let Some(ref pw_name) = decl.target_planeswalker {
                    let pw_id = find_on_battlefield_by_name(state, pw_name)?;
                    AttackTarget::Planeswalker(pw_id)
                } else {
                    // Default: attack the first non-active player, sorted alphabetically by
                    // player name for determinism in multiplayer (3+ player) games. In a
                    // 2-player game there is only one opponent so order is irrelevant.
                    // Note: uses find_on_battlefield_by_name for the creature; see that
                    // function's doc comment about duplicate-name limitations.
                    let mut opponents: Vec<(&str, PlayerId)> = players
                        .iter()
                        .filter(|(_, &pid)| pid != player)
                        .map(|(name, &pid)| (name.as_str(), pid))
                        .collect();
                    opponents.sort_by_key(|(name, _)| *name);
                    let target_pid = opponents.into_iter().next().map(|(_, pid)| pid)?;
                    AttackTarget::Player(target_pid)
                };
                atk_pairs.push((obj_id, target));
            }
            // CR 702.154a: Resolve enlist declarations to (attacker_id, enlisted_id) pairs.
            let mut enlist_choices: Vec<(crate::state::ObjectId, crate::state::ObjectId)> =
                Vec::new();
            for edecl in enlist_decls {
                let attacker_id = find_on_battlefield(state, player, &edecl.attacker)?;
                let enlisted_id = find_on_battlefield_by_name(state, &edecl.enlisted)?;
                enlist_choices.push((attacker_id, enlisted_id));
            }
            Some(Command::DeclareAttackers {
                player,
                attackers: atk_pairs,
                enlist_choices,
            })
        }
        // CR 509.1: Declare blockers. Resolve creature names to ObjectIds on the
        // battlefield. The blocker is controlled by the declaring player; the attacker
        // may be controlled by any player.
        "declare_blockers" => {
            let mut blk_pairs: Vec<(crate::state::ObjectId, crate::state::ObjectId)> = Vec::new();
            for decl in blockers_decl {
                let blocker_id = find_on_battlefield(state, player, &decl.card)?;
                let attacker_id = find_on_battlefield_by_name(state, &decl.blocking)?;
                blk_pairs.push((blocker_id, attacker_id));
            }
            Some(Command::DeclareBlockers {
                player,
                blockers: blk_pairs,
            })
        }
        "concede" => Some(Command::Concede { player }),
        "return_commander_to_command_zone" => {
            // Find the commander by card name in graveyard or exile.
            let card_name = card_name?;
            let obj_id = state.objects.iter().find_map(|(&id, obj)| {
                if obj.characteristics.name == card_name
                    && obj.owner == player
                    && (matches!(obj.zone, ZoneId::Graveyard(_)) || obj.zone == ZoneId::Exile)
                {
                    Some(id)
                } else {
                    None
                }
            })?;
            Some(Command::ReturnCommanderToCommandZone {
                player,
                object_id: obj_id,
            })
        }
        "leave_commander_in_zone" => {
            // Find the commander by card name in graveyard or exile.
            let card_name = card_name?;
            let obj_id = state.objects.iter().find_map(|(&id, obj)| {
                if obj.characteristics.name == card_name
                    && obj.owner == player
                    && (matches!(obj.zone, ZoneId::Graveyard(_)) || obj.zone == ZoneId::Exile)
                {
                    Some(id)
                } else {
                    None
                }
            })?;
            Some(Command::LeaveCommanderInZone {
                player,
                object_id: obj_id,
            })
        }
        // CR 702.122a: Crew a vehicle by tapping creatures.
        // `card_name` is the vehicle's display name; `crew_creatures` field (reusing
        // `convoke_names` for harness re-use) is a JSON array of crew creature names.
        "crew_vehicle" => {
            let vehicle_id = find_on_battlefield(state, player, card_name?)?;
            let crew_ids: Vec<crate::state::game_object::ObjectId> = convoke_names
                .iter()
                .filter_map(|name| find_on_battlefield(state, player, name.as_str()))
                .collect();
            Some(Command::CrewVehicle {
                player,
                vehicle: vehicle_id,
                crew_creatures: crew_ids,
            })
        }
        // CR 702.171a: Saddle a Mount by tapping creatures.
        // `card_name` is the Mount's display name; the saddling creatures are
        // provided in the `convoke_names` field (reused from Crew pattern).
        "saddle_mount" => {
            let mount_id = find_on_battlefield(state, player, card_name?)?;
            let saddle_ids: Vec<crate::state::game_object::ObjectId> = convoke_names
                .iter()
                .filter_map(|name| find_on_battlefield(state, player, name.as_str()))
                .collect();
            Some(Command::SaddleMount {
                player,
                mount: mount_id,
                saddle_creatures: saddle_ids,
            })
        }
        // CR 702.143a / CR 116.2h: Foretell a card from the player's hand.
        // The player pays {2} and exiles the named card face-down. Legal any time
        // the player has priority during their own turn.
        "foretell_card" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            Some(Command::ForetellCard {
                player,
                card: card_id,
            })
        }
        // CR 702.143a: Cast a foretold card from exile by paying the foretell cost.
        // The card must have been foretold on a prior turn (is_foretold == true,
        // foretold_turn < current turn). Uses cast_with_foretell: true.
        "cast_spell_foretell" => {
            let card_id = find_foretold_in_exile(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Foretell),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.170a / CR 116.2k: Plot a card from the player's hand.
        // The player pays the plot cost and exiles the named card face-up.
        // Legal during the player's own main phase with empty stack.
        "plot_card" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            Some(Command::PlotCard {
                player,
                card: card_id,
            })
        }
        // CR 702.170d: Cast a plotted card from exile without paying its mana cost.
        // The card must have been plotted on a prior turn (is_plotted == true,
        // plotted_turn < current turn). Uses AltCostKind::Plot.
        // Legal during the player's own main phase with empty stack.
        "cast_spell_plot" => {
            let card_id = find_plotted_in_exile(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Plot),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.96a: Cast a spell with overload from the player's hand.
        // The overload cost (an alternative cost) is paid instead of the mana cost.
        // The spell has no targets -- it affects all valid objects.
        "cast_spell_overload" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            // CR 702.96b: Overloaded spells have no targets.
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: vec![],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Overload),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.81a: Cast a spell with retrace from the player's graveyard.
        // The player discards a land card from hand as an additional cost.
        // The spell uses its normal mana cost (retrace is additional, not alternative).
        // After resolution the card returns to the graveyard normally (not exiled).
        "cast_spell_retrace" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            let land_name = discard_land_name?;
            let land_id = find_in_hand(state, player, land_name)?;
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Retrace),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![AdditionalCost::Discard(vec![land_id])],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.133a: Cast a spell with jump-start from the player's graveyard.
        // The player pays the card's normal mana cost PLUS discards a card from hand.
        // Unlike retrace, the discarded card may be any card type (not just a land).
        // The card is exiled when it leaves the stack (resolves, countered, or fizzles).
        "cast_spell_jump_start" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            let discard_name = discard_card_name?;
            let discard_id = find_in_hand(state, player, discard_name)?;
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::JumpStart),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![AdditionalCost::Discard(vec![discard_id])],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.127a: Cast the aftermath half of a split card from the player's graveyard.
        // The aftermath half's mana cost is paid (alternative cost) and the card is exiled
        // when it leaves the stack. The card must have AbilityDefinition::Aftermath.
        "cast_spell_aftermath" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Aftermath),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.160 / CR 718: Cast a spell using its prototype cost from the player's hand.
        // Prototype is NOT an alternative cost (CR 118.9 / 2022-10-14 ruling) — orthogonal
        // to alt_cost and can be combined with alternative costs like Flashback or Escape.
        "cast_spell_prototype" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: true,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.109a: Cast a spell with dash from the player's hand.
        // The dash cost (an alternative cost) is paid instead of the mana cost.
        "cast_spell_dash" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Dash),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.152a: Cast a spell with blitz from the player's hand.
        // The blitz cost (an alternative cost) is paid instead of the mana cost.
        "cast_spell_blitz" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Blitz),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.176a: Cast a spell with impending from the player's hand.
        // The impending cost (an alternative cost) is paid instead of the mana cost.
        "cast_spell_impending" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Impending),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.166a: Cast a spell with bargain from the player's hand, sacrificing
        // an artifact, enchantment, or token as the optional additional cost.
        // Bargain is an additional cost (CR 118.8), not an alternative cost -- the
        // spell's normal mana cost is still paid.
        "cast_spell_bargain" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            let bargain_sac_id =
                bargain_sacrifice_name.and_then(|name| find_on_battlefield(state, player, name));
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: bargain_sac_id
                    .map(|id| vec![crate::state::types::AdditionalCost::Sacrifice(vec![id])])
                    .unwrap_or_default(),
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 701.59a: Cast a spell with collect evidence from the player's hand, exiling
        // cards from the caster's graveyard with total mana value >= N as an additional cost.
        // Collect evidence is an additional cost (CR 118.8), not an alternative cost -- the
        // spell's normal mana cost is still paid in full. Unlike Delve, the exiled cards do
        // NOT reduce the mana cost (CR 701.59a).
        "cast_spell_collect_evidence" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            // CR 701.59a: Resolve each evidence card name to an ObjectId in the caster's graveyard.
            let evidence_ids: Vec<crate::state::game_object::ObjectId> = collect_evidence_names
                .iter()
                .filter_map(|name| find_in_graveyard(state, player, name.as_str()))
                .collect();
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![AdditionalCost::CollectEvidenceExile {
                    cards: evidence_ids,
                }],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.119a: Cast a spell with emerge from the player's hand, sacrificing
        // a creature as part of the emerge alternative cost. The total mana cost is
        // reduced by the sacrificed creature's mana value.
        "cast_spell_emerge" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            let emerge_sac_name = emerge_sacrifice_name?;
            let emerge_sac_id = find_on_battlefield(state, player, emerge_sac_name)?;
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Emerge),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![crate::state::types::AdditionalCost::Sacrifice(vec![
                    emerge_sac_id,
                ])],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.137a: Cast a spell with spectacle from the player's hand.
        // The spectacle cost (an alternative cost) is paid instead of the mana cost.
        // Precondition: an opponent of the casting player must have lost life this turn.
        "cast_spell_spectacle" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Spectacle),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        "cast_spell_surge" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Surge),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.153a: Cast a spell with casualty from the player's hand, optionally
        // sacrificing a creature with power >= N as the casualty additional cost.
        // Casualty is an additional cost (CR 118.8), not an alternative cost -- the
        // spell's normal mana cost is still paid.
        // If `casualty_sacrifice_name` is Some, the named creature is looked up on
        // the battlefield and sacrificed as the additional cost.
        "cast_spell_casualty" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            let casualty_sac_id =
                casualty_sacrifice_name.and_then(|name| find_on_battlefield(state, player, name));
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: casualty_sac_id
                    .map(|id| vec![crate::state::types::AdditionalCost::Sacrifice(vec![id])])
                    .unwrap_or_default(),
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.132a: Cast a spell with assist from the player's hand. The assist player
        // pays some amount of the generic mana cost from their own mana pool. The caster
        // pays the remainder. Assist is not an alternative cost — the caster still pays
        // any colored mana pips plus whatever generic remains after assist.
        // `assist_player_name` identifies the assisting player; `assist_amount` is the
        // number of generic mana they pay.
        "cast_spell_assist" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            let assist_pid = assist_player_name.and_then(|name| players.get(name).copied());
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: if let Some(pid) = assist_pid {
                    vec![AdditionalCost::Assist {
                        player: pid,
                        amount: assist_amount,
                    }]
                } else {
                    vec![]
                },
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.56a: Cast a spell with the replicate additional cost paid N times.
        // `replicate_count` is the number of times the replicate cost is paid.
        // Each payment adds the replicate cost to the total mana cost.
        // If `replicate_count > 0`, a `ReplicateTrigger` is placed on the stack that
        // creates N copies of the original spell on resolution.
        "cast_spell_replicate" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: if replicate_count > 0 {
                    vec![AdditionalCost::Replicate {
                        count: replicate_count,
                    }]
                } else {
                    vec![]
                },
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.47a: Cast a spell with splice cards declared.
        // `splice_card_names` lists the names of cards in the caster's hand to splice
        // onto the spell. Each named card must have the Splice ability and be in hand.
        "cast_spell_splice" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            // Resolve each splice card name to an ObjectId in the caster's hand.
            // Fail loudly if any declared splice card is not found — silently dropping
            // it would let a misspelled or absent card go undetected, causing the test
            // to pass for the wrong reason.
            let mut splice_ids = Vec::new();
            for name in splice_card_names {
                let id = find_in_hand(state, player, name.as_str())
                    .unwrap_or_else(|| panic!("splice card '{name}' not found in hand"));
                splice_ids.push(id);
            }
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: if splice_ids.is_empty() {
                    vec![]
                } else {
                    vec![AdditionalCost::Splice { cards: splice_ids }]
                },
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.148a: Cast with Cleave — pay the cleave cost to remove bracketed text.
        "cast_spell_cleave" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: vec![],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Cleave),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.140a: Cast a creature spell using its mutate alternative cost, merging it
        // with a target non-Human creature the caster owns. `target_creature_name` names the
        // target creature on the battlefield. `mutate_on_top` controls whether the mutating
        // spell becomes the topmost component (true) or the bottom component (false).
        // On legal merge the spell does not enter the battlefield separately (CR 729.2b).
        "cast_spell_mutate" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            // Resolve mutate target from target_creature_name (find non-Human creature on battlefield).
            let target_id =
                target_creature_name.and_then(|name| find_on_battlefield(state, player, name))?;
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: vec![],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Mutate),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![AdditionalCost::Mutate {
                    target: target_id,
                    on_top: mutate_on_top,
                }],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.42a: Cast a modal spell with the entwine additional cost paid.
        // When entwine_paid = true, all modes of the spell are chosen and the entwine
        // cost is added to the total mana cost. The spell must have KeywordAbility::Entwine.
        "cast_spell_entwine" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![AdditionalCost::Entwine],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.120a: Cast a modal spell with escalate additional cost paid.
        // `escalate_modes` is the number of extra modes beyond the first. The escalate
        // cost is multiplied by this count and added to the total mana cost.
        // Mode 0 plus `escalate_modes` additional modes execute at resolution.
        "cast_spell_escalate" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: if escalate_modes > 0 {
                    vec![AdditionalCost::EscalateModes {
                        count: escalate_modes,
                    }]
                } else {
                    vec![]
                },
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.82a: Cast with Devour -- sacrifice creatures as an ETB replacement effect.
        // `convoke_names` (reused parameter slot) lists the names of creatures on the
        // battlefield controlled by the caster to sacrifice. Empty list = no sacrifice (devour 0).
        // The sacrifice and counter placement happen at resolution (ETB replacement), not here.
        "cast_spell_devour" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            // Resolve each devour creature name to an ObjectId on the battlefield.
            let devour_ids: Vec<crate::state::game_object::ObjectId> = convoke_names
                .iter()
                .filter_map(|name| find_on_battlefield(state, player, name.as_str()))
                .collect();
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: if devour_ids.is_empty() {
                    vec![]
                } else {
                    vec![crate::state::types::AdditionalCost::Sacrifice(devour_ids)]
                },
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 700.2a / 601.2b: Cast a modal spell with explicit mode indices chosen.
        // `modes_chosen` specifies which mode indices (0-indexed) to execute at resolution.
        // For "choose one" spells: exactly one index (e.g., [0], [1], [2]).
        // For "choose two" spells: exactly two indices (e.g., [0, 2]).
        // For "choose up to N": between 1 and N indices.
        "cast_spell_modal" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: modes_chosen.clone(),
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.102a: Cast a fused split card from hand, paying the combined mana cost
        // of both halves (CR 702.102c). At resolution, the left half's effect executes
        // first, then the right half's (CR 702.102d). Card must be in the caster's hand
        // and must have KeywordAbility::Fuse and AbilityDefinition::Fuse.
        "cast_spell_fuse" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![AdditionalCost::Fuse],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.157a: Cast a creature spell with the squad additional cost paid N times.
        // `squad_count` is the number of times the squad cost is paid (from the action).
        // Each payment adds the squad cost (from AbilityDefinition::Squad { cost }) to the
        // total mana cost. On ETB, a SquadTrigger creates N token copies of the creature.
        "cast_spell_squad" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: if squad_count > 0 {
                    vec![AdditionalCost::Squad { count: squad_count }]
                } else {
                    vec![]
                },
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.175a: Cast a creature spell with the offspring additional cost paid.
        // Sets `offspring_paid: true` on CastSpell. On ETB, an OffspringTrigger creates
        // 1 token copy of the creature except it's 1/1.
        "cast_spell_offspring" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: None,
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![AdditionalCost::Offspring],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.37a / CR 702.168a: Cast a spell face-down via Morph, Megamorph, or Disguise.
        // The card is cast from the player's hand for {3} (the morph cost). The face_down_kind
        // field carries which variant (Morph/Megamorph/Disguise) is used.
        // Action: `card_name` = card to morph-cast. Optional field `face_down_kind` in the
        // script action (defaults to "morph" if not specified).
        "cast_spell_morph" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            // Determine which face-down kind from card definition (auto-detect).
            let kind = {
                let registry = state.card_registry.clone();
                let cid = crate::testing::replay_harness::card_name_to_id(card_name?);
                let def = registry.get(cid);
                if let Some(d) = def {
                    if d.abilities
                        .iter()
                        .any(|a| matches!(a, AbilityDefinition::Disguise { .. }))
                    {
                        FaceDownKind::Disguise
                    } else if d
                        .abilities
                        .iter()
                        .any(|a| matches!(a, AbilityDefinition::Megamorph { .. }))
                    {
                        FaceDownKind::Megamorph
                    } else {
                        FaceDownKind::Morph
                    }
                } else {
                    FaceDownKind::Morph
                }
            };
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: vec![],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Morph),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: Some(kind),
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 118.9 / Commander 2020 cycle: Cast a spell from hand without paying its mana cost,
        // conditional on controlling a commander on the battlefield.
        // 2020-04-17 ruling: any commander (any player's) satisfies the condition.
        // Action JSON fields:
        //   `card_name`: the card to cast from hand
        //   `targets`: optional target list
        "cast_spell_commander_free" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players);
            Some(Command::CastSpell {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::CommanderFreeCast),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })
        }
        // CR 702.37e / CR 702.168d / CR 701.40b: Turn a face-down permanent face up.
        // This is a special action (CR 116.2b) — does NOT use the stack.
        // Action JSON fields:
        //   `card_name`: display name of the face-down permanent (to find by card name)
        // The turn-face-up method is auto-detected from the card definition:
        //   - If the card has Disguise -> DisguiseCost
        //   - If the card has Morph or Megamorph -> MorphCost (default)
        //   - If the card has neither -> ManaCost (manifest/cloak case)
        "turn_face_up" => {
            // Find the face-down permanent controlled by the player.
            // `card_name` is used to identify which face-down permanent to flip.
            let perm_id = find_on_battlefield(state, player, card_name?)?;
            // Auto-detect turn-face-up method from card definition.
            let method = {
                let registry = state.card_registry.clone();
                let cid = card_name_to_id(card_name?);
                let def = registry.get(cid);
                if let Some(d) = def {
                    if d.abilities
                        .iter()
                        .any(|a| matches!(a, AbilityDefinition::Disguise { .. }))
                    {
                        TurnFaceUpMethod::DisguiseCost
                    } else if d.abilities.iter().any(|a| {
                        matches!(
                            a,
                            AbilityDefinition::Morph { .. } | AbilityDefinition::Megamorph { .. }
                        )
                    }) {
                        TurnFaceUpMethod::MorphCost
                    } else {
                        // Manifested/cloaked: pay mana cost
                        TurnFaceUpMethod::ManaCost
                    }
                } else {
                    TurnFaceUpMethod::MorphCost // default
                }
            };
            Some(Command::TurnFaceUp {
                player,
                permanent: perm_id,
                method,
            })
        }
        // CR 701.23a: Document a library search result in a script.
        // The engine resolves SearchLibrary effects deterministically (minimum ObjectId
        // matching the filter). This action is a documentation marker only — no Command
        // is issued. When M10 adds interactive search (Command::SelectLibraryCard),
        // this arm should issue that Command instead.
        "search_library" => None,
        // CR 701.54a: Manually trigger "the Ring tempts you" for the given player.
        // Used in scripts to test ring-temptation directly (without casting a spell).
        // Schema: { "action_type": "ring_tempts_you", "priority_player": "p1" }
        // Deterministic fallback in the engine picks the ring-bearer automatically (lowest ObjectId creature).
        "ring_tempts_you" => Some(Command::TheRingTemptsYou { player }),
        // CR 701.49a-c: Manually trigger a venture into the dungeon for the given player.
        // Used in scripts to test dungeon progression directly (without casting a venture spell).
        // Schema: { "action_type": "venture_into_dungeon", "priority_player": "p1" }
        // Deterministic fallback in the engine picks the dungeon/room automatically.
        "venture_into_dungeon" => Some(Command::VentureIntoDungeon { player }),
        _ => {
            // Unrecognized action — skip without error.
            None
        }
    }
}
/// Convert a card's display name to its canonical [`CardId`] (kebab-case, lowercase).
///
/// Examples:
///   "Lightning Bolt" → `CardId("lightning-bolt")`
///   "Sol Ring"       → `CardId("sol-ring")`
///   "Swords to Plowshares" → `CardId("swords-to-plowshares")`
pub fn card_name_to_id(name: &str) -> CardId {
    let id = name
        .to_lowercase()
        .replace(" // ", "-") // split card names: "Cut // Ribbons" → "cut-ribbons"
        .replace(' ', "-")
        .replace(['\'', ','], "")
        .replace("--", "-"); // avoid double-dashes from punctuation
    CardId(id)
}
/// Map a phase/step string from a script to the engine's [`Step`] enum.
pub fn parse_step(phase: &str) -> Step {
    match phase {
        "untap" => Step::Untap,
        "upkeep" | "beginning_of_upkeep" => Step::Upkeep,
        "draw" | "draw_step" => Step::Draw,
        "precombat_main" | "main1" | "pre_combat_main" => Step::PreCombatMain,
        "beginning_of_combat" | "begin_combat" => Step::BeginningOfCombat,
        "declare_attackers" | "declare_attackers_step" => Step::DeclareAttackers,
        "declare_blockers" | "declare_blockers_step" => Step::DeclareBlockers,
        "combat_damage" | "combat_damage_step" => Step::CombatDamage,
        "first_strike_damage" | "first_strike_damage_step" => Step::FirstStrikeDamage,
        "end_of_combat" | "combat_end" => Step::EndOfCombat,
        "postcombat_main" | "main2" | "post_combat_main" => Step::PostCombatMain,
        "end" | "end_step" | "ending_step" => Step::End,
        "cleanup" => Step::Cleanup,
        _ => Step::PreCombatMain, // Default to main phase.
    }
}
/// Map a counter type string to the engine's [`CounterType`] enum.
pub fn parse_counter_type(s: &str) -> Option<CounterType> {
    match s.to_lowercase().as_str() {
        "+1/+1" | "plus_one_plus_one" | "plus1plus1" => Some(CounterType::PlusOnePlusOne),
        "-1/-1" | "minus_one_minus_one" | "minus1minus1" => Some(CounterType::MinusOneMinusOne),
        "loyalty" => Some(CounterType::Loyalty),
        "poison" => Some(CounterType::Poison),
        "charge" => Some(CounterType::Charge),
        "time" => Some(CounterType::Time),
        _ => None,
    }
}
/// Enrich an [`ObjectSpec`] with card type, mana cost, keyword, and mana-ability
/// information from the card's definition, if available.
///
/// This is necessary because `ObjectSpec::card(owner, name)` creates a minimal
/// object with no characteristics — the harness uses actual card definitions to
/// ensure `PlayLand`, instant-speed checks, and `TapForMana` work correctly.
pub fn enrich_spec_from_def(
    mut spec: ObjectSpec,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    let Some(def) = defs.get(&spec.name) else {
        return spec;
    };
    // Apply card types (Land, Instant, Sorcery, Artifact, etc.)
    spec.card_types = def.types.card_types.iter().cloned().collect();
    // Apply supertypes (Legendary, Basic, etc.)
    if !def.types.supertypes.is_empty() {
        spec.supertypes = def.types.supertypes.iter().cloned().collect();
    }
    // Apply subtypes (Aura, Equipment, Human, etc.) so that SBA checks and
    // the Enchant restriction enforcement can identify the object type.
    if !def.types.subtypes.is_empty() {
        spec.subtypes = def.types.subtypes.iter().cloned().collect();
    }
    // Apply mana cost (for cost-payment validation at cast time).
    spec.mana_cost = def.mana_cost.clone();
    // Apply oracle text for display.
    if !def.oracle_text.is_empty() {
        spec.rules_text = def.oracle_text.clone();
    }
    // CR 204: Color indicator overrides mana-cost-derived colors.
    if let Some(ref ci) = def.color_indicator {
        spec.colors = ci.clone();
    } else if spec.colors.is_empty() {
        // Derive colors from mana cost (CR 202.2) — only if not already set.
        if let Some(ref cost) = def.mana_cost {
            let mut colors = Vec::new();
            if cost.white > 0 {
                colors.push(Color::White);
            }
            if cost.blue > 0 {
                colors.push(Color::Blue);
            }
            if cost.black > 0 {
                colors.push(Color::Black);
            }
            if cost.red > 0 {
                colors.push(Color::Red);
            }
            if cost.green > 0 {
                colors.push(Color::Green);
            }
            spec.colors = colors;
        }
    }
    // Apply printed power/toughness for creatures.
    // This allows EffectAmount::PowerOf / ToughnessOf to read correct values.
    if def.power.is_some() {
        spec.power = def.power;
    }
    if def.toughness.is_some() {
        spec.toughness = def.toughness;
    }
    // CR 306.5a: Apply starting loyalty for planeswalkers.
    if let Some(loyalty) = def.starting_loyalty {
        spec.loyalty = Some(loyalty as i32);
    }
    // Apply keyword abilities (Haste, Vigilance, Hexproof, etc.)
    for ability in &def.abilities {
        if let AbilityDefinition::Keyword(kw) = ability {
            spec = spec.with_keyword(kw.clone());
        }
    }
    // Convert simple tap-for-mana activated abilities into mana abilities.
    // This covers basic lands and any rock with `{T}: Add {N mana}`.
    // Multi-step costs (e.g. Evolving Wilds's tap+sacrifice) are intentionally
    // excluded — those are activated abilities, not mana abilities.
    for ability in &def.abilities {
        if let AbilityDefinition::Activated { cost, effect, .. } = ability {
            if matches!(cost, Cost::Tap) {
                if let Some(ma) = try_as_tap_mana_ability(effect) {
                    spec = spec.with_mana_ability(ma);
                }
            }
        }
    }
    // Populate non-mana activated abilities into characteristics.activated_abilities.
    // This is required so that Command::ActivateAbility can look up the ability by index.
    for ability in &def.abilities {
        if let AbilityDefinition::Activated {
            cost,
            effect,
            timing_restriction,
            targets: ab_targets,
            activation_condition,
            activation_zone,
            once_per_turn,
            ..
        } = ability
        {
            // Skip ALL tap-for-mana abilities (fixed-mana, any-color, and pain-land
            // Sequence variants). These are registered as ManaAbility above via
            // try_as_tap_mana_ability. Including them here would shift ability_index.
            let is_tap_mana_ability = matches!(cost, Cost::Tap)
                && (matches!(
                    effect,
                    Effect::AddMana { .. }
                        | Effect::AddManaAnyColor { .. }
                        | Effect::AddManaScaled { .. }
                        | Effect::AddManaFilterChoice { .. }
                        | Effect::AddManaMatchingType { .. }
                ) || try_as_tap_mana_ability(effect).is_some());
            if !is_tap_mana_ability {
                let activation_cost = cost_to_activation_cost(cost);
                let ab = ActivatedAbility {
                    cost: activation_cost,
                    description: String::new(),
                    effect: Some(effect.clone()),
                    // CR 602.5d: Propagate timing restriction so sorcery-speed abilities
                    // (e.g., Equip) are enforced in handle_activate_ability.
                    sorcery_speed: matches!(
                        timing_restriction,
                        Some(TimingRestriction::SorcerySpeed)
                    ),
                    targets: ab_targets.clone(),
                    // CR 602.5b: Propagate activation condition ("activate only if ...").
                    activation_condition: activation_condition.clone(),
                    // CR 602.2: Propagate activation zone for graveyard-activated abilities.
                    activation_zone: activation_zone.clone(),
                    // CR 602.5b: Propagate once-per-turn restriction.
                    once_per_turn: *once_per_turn,
                };
                spec = spec.with_activated_ability(ab);
            }
        }
    }
    // CR 702.72: AbilityDefinition::Champion { .. } adds KeywordAbility::Champion marker.
    // The champion filter is looked up at trigger time from the card registry.
    for ability in &def.abilities {
        if let AbilityDefinition::Champion { .. } = ability {
            spec = spec.with_keyword(KeywordAbility::Champion);
        }
    }
    // CR 702.95: AbilityDefinition::Soulbond { .. } adds KeywordAbility::Soulbond marker.
    // The soulbond grants are looked up at SoulbondTrigger resolution from the card registry.
    for ability in &def.abilities {
        if let AbilityDefinition::Soulbond { .. } = ability {
            spec = spec.with_keyword(KeywordAbility::Soulbond);
        }
    }
    // CR 702.99a: AbilityDefinition::Cipher adds KeywordAbility::Cipher marker.
    // The resolution path checks obj.characteristics.keywords.contains(&KeywordAbility::Cipher)
    // to decide whether to offer cipher encoding. Without this propagation, cards placed
    // through the harness never trigger cipher encoding and go to the graveyard instead.
    for ability in &def.abilities {
        if let AbilityDefinition::Cipher = ability {
            spec = spec.with_keyword(KeywordAbility::Cipher);
        }
    }
    // CR 702.174a: AbilityDefinition::Gift adds KeywordAbility::Gift marker.
    // The casting validation checks chars.keywords.contains(&KeywordAbility::Gift) to confirm
    // the spell supports a gift opponent choice. Without this propagation, gift spells placed
    // through the harness reject the gift_opponent field with "spell does not have gift".
    for ability in &def.abilities {
        if let AbilityDefinition::Gift { .. } = ability {
            spec = spec.with_keyword(KeywordAbility::Gift);
        }
    }
    // CR 702.151a: AbilityDefinition::Reconfigure expands into TWO activated abilities:
    // 1. Attach: "[Cost]: Attach this permanent to another target creature you control.
    //    Activate only as a sorcery." (uses AttachEquipment effect, same as Equip)
    // 2. Unattach: "[Cost]: Unattach this permanent. Activate only as a sorcery."
    //    (uses DetachEquipment effect; validated in handle_activate_ability)
    //
    // Also adds KeywordAbility::Reconfigure marker for presence-checking.
    for ability in &def.abilities {
        if let AbilityDefinition::Reconfigure { cost } = ability {
            spec = spec.with_keyword(KeywordAbility::Reconfigure);
            // Ability 1: Attach to a target creature you control (sorcery speed).
            let attach_ab = ActivatedAbility {
                targets: vec![],
                cost: cost_to_activation_cost(&Cost::Mana(cost.clone())),
                description: "Reconfigure (CR 702.151a): Attach to target creature you control."
                    .to_string(),
                effect: Some(Effect::AttachEquipment {
                    equipment: CardEffectTarget::Source,
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                }),
                sorcery_speed: true,
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            };
            spec = spec.with_activated_ability(attach_ab);
            // Ability 2: Unattach from the equipped creature (sorcery speed).
            // Activation restriction "only if attached" is enforced in handle_activate_ability.
            let detach_ab = ActivatedAbility {
                targets: vec![],
                cost: cost_to_activation_cost(&Cost::Mana(cost.clone())),
                description: "Reconfigure (CR 702.151a): Unattach from equipped creature."
                    .to_string(),
                effect: Some(Effect::DetachEquipment {
                    equipment: CardEffectTarget::Source,
                }),
                sorcery_speed: true,
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            };
            spec = spec.with_activated_ability(detach_ab);
        }
    }
    // CR 702.107a: Expand AbilityDefinition::Outlast into an ActivatedAbility.
    // "Outlast [cost]" means "[Cost], {T}: Put a +1/+1 counter on this creature.
    // Activate only as a sorcery."
    for ability in &def.abilities {
        if let AbilityDefinition::Outlast { cost } = ability {
            let ab = ActivatedAbility {
                targets: vec![],
                cost: ActivationCost {
                    requires_tap: true,
                    mana_cost: Some(cost.clone()),
                    sacrifice_self: false,
                    discard_card: false,
                    discard_self: false,
                    forage: false,
                    sacrifice_filter: None,
                    remove_counter_cost: None,
                    exile_self: false,
                },
                description: "Outlast (CR 702.107a)".to_string(),
                effect: Some(Effect::AddCounter {
                    target: CardEffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                }),
                sorcery_speed: true,
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            };
            spec = spec.with_activated_ability(ab);
        }
    }
    // CR 603.6c / CR 700.4: Convert "When ~ dies" card-definition triggers into
    // runtime TriggeredAbilityDef entries so check_triggers can dispatch them.
    // This covers self-referential dies triggers (e.g. Solemn Simulacrum).
    // CR 700.2b: For modal WhenDies triggers, use mode 0 as the bot fallback effect.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenDies,
            effect,
            modes,
            targets,
            ..
        } = ability
        {
            // CR 700.2b: If modal, pre-select mode 0 as the bot fallback.
            let resolved_effect = if let Some(m) = modes {
                m.modes.first().cloned().unwrap_or_else(|| effect.clone())
            } else {
                effect.clone()
            };
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                trigger_on: TriggerEvent::SelfDies,
                intervening_if: None,
                targets: targets.clone(),
                description: "When ~ dies (CR 700.4)".to_string(),
                effect: Some(resolved_effect),
            });
        }
    }
    // CR 508.1m / CR 508.3a: Convert "Whenever ~ attacks" card-definition triggers into
    // runtime TriggeredAbilityDef entries so check_triggers can dispatch them.
    // This covers self-referential attack triggers (e.g. Audacious Thief).
    // Note: CR 508.4 — creatures put onto the battlefield attacking do NOT trigger
    // "whenever ~ attacks"; they were never declared as attackers. The engine correctly
    // dispatches SelfAttacks only from AttackersDeclared events (not from any other
    // mechanism), so this enrichment path is safe.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenAttacks,
            effect,
            modes,
            ..
        } = ability
        {
            // CR 700.2b: If modal, pre-select mode 0 as the bot fallback.
            let resolved_effect = if let Some(m) = modes {
                m.modes.first().cloned().unwrap_or_else(|| effect.clone())
            } else {
                effect.clone()
            };
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                trigger_on: TriggerEvent::SelfAttacks,
                intervening_if: None,
                targets: vec![],
                description: "Whenever ~ attacks (CR 508.3a)".to_string(),
                effect: Some(resolved_effect),
            });
        }
    }
    // CR 509.1: Convert "Whenever ~ blocks" card-definition triggers into runtime
    // TriggeredAbilityDef entries so check_triggers can dispatch them.
    // This covers self-referential block triggers (e.g. a creature with "Whenever ~ blocks").
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenBlocks,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                trigger_on: TriggerEvent::SelfBlocks,
                intervening_if: None,
                targets: vec![],
                description: "Whenever ~ blocks (CR 509.1)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 510.3a / CR 603.2: Convert "Whenever ~ deals combat damage to a player"
    // card-definition triggers into runtime TriggeredAbilityDef entries so
    // check_triggers can dispatch them via CombatDamageDealt events.
    // intervening_if is None here: Condition and InterveningIf are separate types;
    // conditional combat-damage triggers are rare and deferred.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
            effect,
            modes,
            targets,
            ..
        } = ability
        {
            // CR 700.2b: If modal, pre-select mode 0 as the bot fallback.
            let resolved_effect = if let Some(m) = modes {
                m.modes.first().cloned().unwrap_or_else(|| effect.clone())
            } else {
                effect.clone()
            };
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                trigger_on: TriggerEvent::SelfDealsCombatDamageToPlayer,
                intervening_if: None,
                targets: targets.clone(),
                description: "Whenever ~ deals combat damage to a player (CR 510.3a)".to_string(),
                effect: Some(resolved_effect),
            });
        }
    }
    // CR 207.2c / CR 120.3: Convert "Whenever ~ is dealt damage" (Enrage ability
    // word) card-definition triggers into runtime TriggeredAbilityDef entries so
    // check_triggers can dispatch them via CombatDamageDealt and DamageDealt events.
    // Per ruling 2018-01-19, multiple simultaneous sources trigger only once per creature.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenDealtDamage,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                trigger_on: TriggerEvent::SelfIsDealtDamage,
                intervening_if: None,
                targets: vec![],
                description: "Enrage -- Whenever this creature is dealt damage (CR 207.2c)"
                    .to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 603.2 / CR 102.2: Convert "Whenever an opponent casts a spell"
    // card-definition triggers into runtime TriggeredAbilityDef entries so
    // check_triggers can dispatch them via SpellCast events.
    // The opponent check is done at trigger-collection time in abilities.rs,
    // not here -- this only wires the TriggerEvent.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverOpponentCastsSpell { .. },
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                trigger_on: TriggerEvent::OpponentCastsSpell,
                intervening_if: None,
                targets: vec![],
                description: "Whenever an opponent casts a spell (CR 603.2)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 701.25d: Convert "Whenever you surveil" card-definition triggers into
    // runtime TriggeredAbilityDef entries so check_triggers can dispatch them
    // via Surveilled events.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouSurveil,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                trigger_on: TriggerEvent::ControllerSurveils,
                intervening_if: None,
                targets: vec![],
                description: "Whenever you surveil (CR 701.25d)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 701.50b: Convert "Whenever this creature connives" card-definition triggers
    // into runtime TriggeredAbilityDef entries so check_triggers can dispatch them
    // via Connived events. Fires even if the creature left the battlefield
    // (Psychic Pickpocket ruling, 2022-04-29).
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenConnives,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                trigger_on: TriggerEvent::SourceConnives,
                intervening_if: None,
                targets: vec![],
                description: "Whenever this creature connives (CR 701.50b)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 701.16a: Convert "Whenever you investigate" card-definition triggers into
    // runtime TriggeredAbilityDef entries so check_triggers can dispatch them
    // via Investigated events.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouInvestigate,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                trigger_on: TriggerEvent::ControllerInvestigates,
                intervening_if: None,
                targets: vec![],
                description: "Whenever you investigate (CR 701.16a)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 603.2: Convert "Whenever you cast a spell" card-definition triggers into
    // runtime TriggeredAbilityDef entries so check_triggers can dispatch them
    // via SpellCast events. Covers Inexorable Tide and similar enchantments.
    // The `during_opponent_turn` flag is not enforced here — turn filtering is
    // done at trigger-collection time in abilities.rs via ControllerCastsSpell.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouCastSpell { .. },
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                trigger_on: TriggerEvent::ControllerCastsSpell,
                intervening_if: None,
                targets: vec![],
                description: "Whenever you cast a spell (CR 603.2)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 207.2c / CR 603.2: Convert "Whenever [another] creature [you control] enters"
    // card-definition triggers into runtime TriggeredAbilityDef entries so
    // check_triggers can dispatch them via AnyPermanentEntersBattlefield events.
    //
    // Used by the Alliance ability word (Prosperous Innkeeper, etc.) and similar patterns
    // (Impact Tremors uses the same TriggerCondition but is an enchantment, not a creature).
    // The ETB filter is applied at trigger-collection time in collect_triggers_for_event.
    //
    // `exclude_self` is always true because:
    // - Alliance cards say "another creature" -- the card itself must not trigger itself.
    // - Non-creature sources (e.g. Impact Tremors) can never BE the entering creature,
    //   so exclude_self: true is correct and harmless for non-creature trigger sources.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield { filter },
            effect,
            ..
        } = ability
        {
            let etb_filter = ETBTriggerFilter {
                creature_only: true,
                controller_you: filter
                    .as_ref()
                    .is_some_and(|f| matches!(f.controller, TargetController::You)),
                exclude_self: true,
                // Propagate color filter from TargetFilter to ETBTriggerFilter.
                // e.g., Shadow Alley Denizen: "another black creature you control enters"
                color_filter: filter
                    .as_ref()
                    .and_then(|f| f.colors.as_ref())
                    .and_then(|colors| {
                        if colors.len() == 1 {
                            colors.iter().next().copied()
                        } else {
                            None
                        }
                    }),
            };
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
                // Intervening-if conditions (card_definition::Condition) are a different
                // type from runtime InterveningIf; conversion is deferred. None is safe
                // for all known Alliance cards.
                intervening_if: None,
                description: "Alliance -- Whenever another creature you control enters (CR 207.2c)"
                    .to_string(),
                effect: Some(effect.clone()),
                etb_filter: Some(etb_filter),
                death_filter: None,
                combat_damage_filter: None,
                targets: vec![],
            });
        }
    }
    // CR 702.140d: Convert "Whenever this creature mutates" card-definition triggers
    // into runtime TriggeredAbilityDef entries so check_triggers can dispatch them
    // via CreatureMutated events. Only fires on the merged permanent itself (CR 729.2c).
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenMutates,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                trigger_on: TriggerEvent::SelfMutates,
                intervening_if: None,
                targets: vec![],
                description: "Whenever this creature mutates (CR 702.140d)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // "Whenever this permanent becomes tapped" — fires on any tap event (mana, combat,
    // opponent effects). Used by City of Brass. Maps to TriggerEvent::SelfBecomesTapped
    // which is already dispatched from GameEvent::PermanentTapped in check_triggers.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenSelfBecomesTapped,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                trigger_on: TriggerEvent::SelfBecomesTapped,
                intervening_if: None,
                targets: vec![],
                description: "Whenever this permanent becomes tapped".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 603.10a: Convert "Whenever [another] [nontoken] creature [you control / an opponent controls] dies"
    // card-definition triggers into runtime TriggeredAbilityDef entries so check_triggers can
    // dispatch them via AnyCreatureDies events.
    //
    // The controller filter, exclude_self, and nontoken_only fields are set on DeathTriggerFilter
    // and applied at trigger-collection time in collect_triggers_for_event.
    //
    // CR 603.10a: Convert "Whenever [another] [nontoken] creature [you control / an opponent controls] dies"
    // card-definition triggers into runtime TriggeredAbilityDef entries.
    // The exclude_self and nontoken_only fields from the card def are forwarded directly to
    // DeathTriggerFilter and applied at trigger-collection time.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition:
                TriggerCondition::WheneverCreatureDies {
                    controller,
                    exclude_self,
                    nontoken_only,
                },
            effect,
            ..
        } = ability
        {
            let death_filter = DeathTriggerFilter {
                controller_you: matches!(controller, Some(TargetController::You)),
                controller_opponent: matches!(controller, Some(TargetController::Opponent)),
                exclude_self: *exclude_self,
                nontoken_only: *nontoken_only,
            };
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::AnyCreatureDies,
                intervening_if: None,
                description: "Whenever a creature dies (CR 603.10a)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: Some(death_filter),
                combat_damage_filter: None,
                targets: vec![],
            });
        }
    }
    // CR 508.1m / CR 603.2: Convert "Whenever a creature you control attacks" card-definition
    // triggers into runtime TriggeredAbilityDef entries so check_triggers can dispatch them
    // via AnyCreatureYouControlAttacks events.
    //
    // Controller filtering is applied at trigger-collection time by checking that the attacking
    // creature's controller matches the trigger source's controller.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::AnyCreatureYouControlAttacks,
                intervening_if: None,
                description: "Whenever a creature you control attacks (CR 508.1m)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                targets: vec![],
            });
        }
    }
    // CR 510.3a / CR 603.2: Convert "Whenever a creature you control deals combat damage to a
    // player" card-definition triggers into runtime TriggeredAbilityDef entries so check_triggers
    // can dispatch them via AnyCreatureYouControlDealsCombatDamageToPlayer events.
    //
    // Controller filtering is applied at trigger-collection time by checking that the source
    // creature's controller matches the trigger source's controller.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition:
                TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer { filter },
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer,
                intervening_if: None,
                description:
                    "Whenever a creature you control deals combat damage to a player (CR 510.3a)"
                        .to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: filter.clone(),
                targets: vec![],
            });
        }
    }
    // CR 510.3a / CR 603.2c: Convert "Whenever one or more creatures you control deal combat
    // damage to a player" batch trigger into runtime TriggeredAbilityDef entries.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition:
                TriggerCondition::WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer { filter },
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::AnyCreatureYouControlBatchCombatDamage,
                intervening_if: None,
                description:
                    "Whenever one or more creatures you control deal combat damage to a player (CR 510.3a)"
                        .to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: filter.clone(),
                targets: vec![],
            });
        }
    }
    // CR 510.3a: Convert "Whenever equipped creature deals combat damage to a player" triggers.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::EquippedCreatureDealsCombatDamageToPlayer,
                intervening_if: None,
                description:
                    "Whenever equipped creature deals combat damage to a player (CR 510.3a)"
                        .to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                targets: vec![],
            });
        }
    }
    // CR 510.3a: Convert "Whenever enchanted creature deals damage to a player" triggers.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer { .. },
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::EnchantedCreatureDealsDamageToPlayer,
                intervening_if: None,
                description: "Whenever enchanted creature deals damage to a player (CR 510.3a)"
                    .to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                targets: vec![],
            });
        }
    }
    // CR 510.3a / CR 603.2: Convert "Whenever a creature deals combat damage to one of your
    // opponents" (Edric) triggers into runtime TriggeredAbilityDef entries.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenAnyCreatureDealsCombatDamageToOpponent,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::AnyCreatureDealsCombatDamageToOpponent,
                intervening_if: None,
                description:
                    "Whenever a creature deals combat damage to one of your opponents (CR 510.3a)"
                        .to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                targets: vec![],
            });
        }
    }
    // CR 701.9a: Convert "Whenever you discard a card" triggers.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouDiscard,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::ControllerDiscards,
                intervening_if: None,
                description: "Whenever you discard a card (CR 701.9a)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                targets: vec![],
            });
        }
    }
    // CR 701.9a: Convert "Whenever an opponent discards a card" triggers.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverOpponentDiscards,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::OpponentDiscards,
                intervening_if: None,
                description: "Whenever an opponent discards a card (CR 701.9a)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                targets: vec![],
            });
        }
    }
    // CR 305.1: Convert "Whenever an opponent plays a land" triggers.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverOpponentPlaysLand,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::OpponentPlaysLand,
                intervening_if: None,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                targets: vec![],
                description: "Whenever an opponent plays a land (CR 305.1)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 701.21a: Convert "Whenever you sacrifice a permanent" triggers.
    // player_filter=None → ControllerSacrifices (fires only when controller sacrifices).
    // player_filter=Some(Any) → ControllerSacrifices (any player; filtered at dispatch time).
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouSacrifice { .. },
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::ControllerSacrifices,
                intervening_if: None,
                description: "Whenever you sacrifice a permanent (CR 701.21a)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                targets: vec![],
            });
        }
    }
    // CR 508.1: Convert "Whenever you attack" triggers.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouAttack,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::ControllerAttacks,
                intervening_if: None,
                description: "Whenever you attack (CR 508.1)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                targets: vec![],
            });
        }
    }
    // CR 603.10a: Convert "When ~ leaves the battlefield" triggers.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenLeavesBattlefield,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::SelfLeavesBattlefield,
                intervening_if: None,
                description: "When ~ leaves the battlefield (CR 603.10a)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                targets: vec![],
            });
        }
    }
    // CR 603.2: Convert "Whenever you draw a card" triggers (WheneverYouDrawACard).
    // Maps to ControllerDrawsCard event — dispatched via CardDrawn.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouDrawACard,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::ControllerDrawsCard,
                intervening_if: None,
                description: "Whenever you draw a card (CR 603.2)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                targets: vec![],
            });
        }
    }
    // CR 603.2: Convert "Whenever a player draws a card" triggers (WheneverPlayerDrawsCard).
    // player_filter=None → AnyPlayerDrawsCard, Some(Opponent) → OpponentDrawsCard,
    // Some(You) → ControllerDrawsCard.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverPlayerDrawsCard { player_filter },
            effect,
            ..
        } = ability
        {
            let trigger_on = match player_filter {
                Some(TargetController::Opponent) => TriggerEvent::OpponentDrawsCard,
                Some(TargetController::You) => TriggerEvent::ControllerDrawsCard,
                _ => TriggerEvent::AnyPlayerDrawsCard,
            };
            let desc = match player_filter {
                Some(TargetController::Opponent) => "Whenever an opponent draws a card (CR 603.2)",
                Some(TargetController::You) => "Whenever you draw a card (CR 603.2)",
                _ => "Whenever a player draws a card (CR 603.2)",
            };
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on,
                intervening_if: None,
                description: desc.to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                targets: vec![],
            });
        }
    }
    // CR 603.2 / CR 118.4: Convert "Whenever you gain life" triggers.
    // Maps to ControllerGainsLife — dispatched via LifeGained.
    for ability in &def.abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouGainLife,
            effect,
            ..
        } = ability
        {
            spec = spec.with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::ControllerGainsLife,
                intervening_if: None,
                description: "Whenever you gain life (CR 603.2)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                targets: vec![],
            });
        }
    }
    // CR 603.2: WhenYouCastThisSpell is dispatched directly from the SpellCast arm
    // in check_triggers using the CardDef ability_index. No TriggeredAbilityDef needed.
    spec
}
// ── Private helpers ───────────────────────────────────────────────────────────
fn resolve_targets(
    targets: &[ActionTarget],
    state: &GameState,
    players: &HashMap<String, PlayerId>,
) -> Vec<crate::state::Target> {
    targets
        .iter()
        .filter_map(|t| {
            match t.target_type.as_str() {
                "player" => {
                    let pname = t.player.as_deref()?;
                    players
                        .get(pname)
                        .map(|&pid| crate::state::Target::Player(pid))
                }
                "spell" => {
                    let cname = t.card.as_deref()?;
                    // Look for the named object on the stack.
                    let obj_id = state.objects.iter().find_map(|(&id, obj)| {
                        if obj.characteristics.name == cname && obj.zone == ZoneId::Stack {
                            Some(id)
                        } else {
                            None
                        }
                    })?;
                    Some(crate::state::Target::Object(obj_id))
                }
                "permanent" | "creature" | "artifact" | "enchantment" | "card" => {
                    let cname = t.card.as_deref()?;
                    // Look for the named permanent on the battlefield.
                    let obj_id = state.objects.iter().find_map(|(&id, obj)| {
                        if obj.characteristics.name == cname && obj.zone == ZoneId::Battlefield {
                            Some(id)
                        } else {
                            None
                        }
                    })?;
                    Some(crate::state::Target::Object(obj_id))
                }
                _ => None,
            }
        })
        .collect()
}
fn find_in_hand(state: &GameState, player: PlayerId, name: &str) -> Option<crate::state::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == ZoneId::Hand(player) {
            Some(id)
        } else {
            None
        }
    })
}
/// CR 702.49d: Find a named card in a player's command zone (for commander ninjutsu).
fn find_in_command_zone(
    state: &GameState,
    player: PlayerId,
    name: &str,
) -> Option<crate::state::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == ZoneId::Command(player) {
            Some(id)
        } else {
            None
        }
    })
}
/// CR 702.34a: Find a named card in a player's graveyard (for flashback casting).
fn find_in_graveyard(
    state: &GameState,
    player: PlayerId,
    name: &str,
) -> Option<crate::state::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == ZoneId::Graveyard(player) {
            Some(id)
        } else {
            None
        }
    })
}
fn find_in_exile(
    state: &GameState,
    _player: PlayerId,
    name: &str,
) -> Option<crate::state::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == ZoneId::Exile {
            Some(id)
        } else {
            None
        }
    })
}
/// CR 702.143a: Find a named foretold card in exile owned by the given player.
///
/// Foretell cards are in ZoneId::Exile with is_foretold == true. Unlike general
/// exile (which is a shared zone), foretold cards are filtered by owner.
fn find_foretold_in_exile(
    state: &GameState,
    player: PlayerId,
    name: &str,
) -> Option<crate::state::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name
            && obj.zone == ZoneId::Exile
            && obj.designations.contains(Designations::FORETOLD)
            && obj.owner == player
        {
            Some(id)
        } else {
            None
        }
    })
}
/// CR 702.170a: Find a named plotted card in exile owned by the given player.
///
/// Plotted cards are in ZoneId::Exile with is_plotted == true. Unlike general
/// exile (which is a shared zone), plotted cards are filtered by owner.
fn find_plotted_in_exile(
    state: &GameState,
    player: PlayerId,
    name: &str,
) -> Option<crate::state::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name
            && obj.zone == ZoneId::Exile
            && obj.is_plotted
            && obj.owner == player
        {
            Some(id)
        } else {
            None
        }
    })
}
fn find_on_battlefield(
    state: &GameState,
    controller: PlayerId,
    name: &str,
) -> Option<crate::state::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name
            && obj.zone == ZoneId::Battlefield
            && obj.controller == controller
        {
            Some(id)
        } else {
            None
        }
    })
}
/// CR 509.1: Find any permanent on the battlefield by name, regardless of controller.
///
/// Used when declaring blockers — the attacker being blocked is controlled by an
/// opponent, so we cannot filter by the declaring player's controller field.
///
/// **Duplicate-name limitation**: if multiple permanents share the same card name,
/// the one with the lowest `ObjectId` is returned (because `state.objects` is an
/// `OrdMap<ObjectId, GameObject>` and `find_map` returns the first match). Use unique
/// card names in test scripts to avoid ambiguity. Adding an `index` field to
/// `BlockerDeclaration` (e.g. `"index": 1` for the second copy) is a future improvement.
fn find_on_battlefield_by_name(state: &GameState, name: &str) -> Option<crate::state::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == ZoneId::Battlefield {
            Some(id)
        } else {
            None
        }
    })
}
/// If `effect` is `AddMana` with exactly one non-zero single-color entry,
/// return a corresponding `ManaAbility::tap_for`. Returns `None` otherwise.
///
/// This covers all 5 basic land colors (produces exactly 1 mana of one color).
/// Sol Ring ({T}: Add {CC}) produces 2 colorless — handled via ActivateAbility
/// in scripts instead of TapForMana.
fn try_as_tap_mana_ability(effect: &Effect) -> Option<ManaAbility> {
    // Simple: {T}: Add {mana}
    if let Effect::AddMana { mana, .. } = effect {
        return mana_pool_to_ability(mana, 0);
    }
    // Any color: {T}: Add one mana of any color
    if matches!(effect, Effect::AddManaAnyColor { .. }) {
        return Some(ManaAbility {
            produces: im::OrdMap::new(),
            requires_tap: true,
            sacrifice_self: false,
            any_color: true,
            damage_to_controller: 0,
        });
    }
    // Filter land pattern: {Hybrid},{T}: AddManaFilterChoice { color_a, color_b }
    // Produces 1 of color_a + 1 of color_b (middle option of 3 choices).
    if let Effect::AddManaFilterChoice {
        color_a, color_b, ..
    } = effect
    {
        let mut produces = im::OrdMap::new();
        produces.insert(*color_a, 1u32);
        *produces.entry(*color_b).or_insert(0) += 1;
        return Some(ManaAbility {
            produces,
            requires_tap: true,
            sacrifice_self: false,
            any_color: false,
            damage_to_controller: 0,
        });
    }
    // Scaled mana: {T}: AddManaScaled { color, count }
    // Registers with produces={color: 1} as a marker; actual production is dynamic.
    if let Effect::AddManaScaled { color, .. } = effect {
        let mut p = im::OrdMap::new();
        p.insert(*color, 1u32);
        return Some(ManaAbility {
            produces: p,
            requires_tap: true,
            sacrifice_self: false,
            any_color: false,
            damage_to_controller: 0,
        });
    }
    // Pain land pattern: {T}: Sequence([AddMana, DealDamage{Controller, Fixed(n)}])
    if let Effect::Sequence(effects) = effect {
        if effects.len() == 2 {
            if let (
                Effect::AddMana { mana, .. },
                Effect::DealDamage {
                    target: CardEffectTarget::Controller,
                    amount: EffectAmount::Fixed(dmg),
                },
            ) = (&effects[0], &effects[1])
            {
                return mana_pool_to_ability(mana, *dmg as u32);
            }
        }
    }
    None
}
/// Convert a ManaPool into a ManaAbility, returning None if no mana is produced.
fn mana_pool_to_ability(
    mana: &crate::state::player::ManaPool,
    damage_to_controller: u32,
) -> Option<ManaAbility> {
    let color_amounts = [
        (ManaColor::White, mana.white),
        (ManaColor::Blue, mana.blue),
        (ManaColor::Black, mana.black),
        (ManaColor::Red, mana.red),
        (ManaColor::Green, mana.green),
        (ManaColor::Colorless, mana.colorless),
    ];
    let non_zero: Vec<_> = color_amounts
        .iter()
        .filter(|(_, amount)| *amount > 0)
        .collect();
    if non_zero.is_empty() {
        return None;
    }
    let mut produces = OrdMap::new();
    for (color, amount) in &non_zero {
        produces.insert(*color, *amount);
    }
    Some(ManaAbility {
        produces,
        requires_tap: true,
        sacrifice_self: false,
        any_color: false,
        damage_to_controller,
    })
}
/// Convert a card-definition [`Cost`] into an [`ActivationCost`] for object characteristics.
///
/// Handles `Tap`, `Mana`, `Sacrifice`, `DiscardCard`, and `Sequence` (recursively).
/// Unrecognised cost components (`PayLife`) are silently ignored.
fn cost_to_activation_cost(cost: &Cost) -> ActivationCost {
    let mut ac = ActivationCost::default();
    flatten_cost_into(cost, &mut ac);
    ac
}
fn flatten_cost_into(cost: &Cost, ac: &mut ActivationCost) {
    match cost {
        Cost::Tap => ac.requires_tap = true,
        Cost::Mana(m) => ac.mana_cost = Some(m.clone()),
        Cost::SacrificeSelf => ac.sacrifice_self = true,
        Cost::Sacrifice(filter) => {
            // Convert TargetFilter to SacrificeFilter.
            // has_chosen_subtype: dynamic check against the activating permanent's chosen_creature_type.
            // Recorded as SacrificeFilter::Creature with chosen_subtype_required flag (validated later).
            let sac_filter = if filter.has_chosen_subtype {
                // Creature of the chosen type — validated dynamically in validate_sacrifice_cost.
                SacrificeFilter::CreatureOfChosenType
            } else if let Some(ref subtype) = filter.has_subtype {
                SacrificeFilter::Subtype(subtype.clone())
            } else {
                match filter.has_card_type {
                    Some(CardType::Creature) => SacrificeFilter::Creature,
                    Some(CardType::Land) => SacrificeFilter::Land,
                    Some(CardType::Artifact) => SacrificeFilter::Artifact,
                    other => {
                        debug_assert!(
                            other.is_none(),
                            "unhandled card type in Cost::Sacrifice filter: {:?}",
                            other
                        );
                        SacrificeFilter::Creature // conservative fallback
                    }
                }
            };
            ac.sacrifice_filter = Some(sac_filter);
        }
        Cost::Sequence(costs) => costs.iter().for_each(|c| flatten_cost_into(c, ac)),
        Cost::DiscardCard => ac.discard_card = true,
        Cost::DiscardSelf => ac.discard_self = true,
        Cost::Forage => ac.forage = true,
        Cost::ExileSelf => ac.exile_self = true,
        Cost::PayLife(_) => {} // no ActivationCost representation yet
        Cost::RemoveCounter { counter, count } => {
            ac.remove_counter_cost = Some((counter.clone(), *count));
        }
    }
}
fn parse_mana_color(s: &str) -> Option<ManaColor> {
    match s.to_lowercase().as_str() {
        "white" | "w" => Some(ManaColor::White),
        "blue" | "u" => Some(ManaColor::Blue),
        "black" | "b" => Some(ManaColor::Black),
        "red" | "r" => Some(ManaColor::Red),
        "green" | "g" => Some(ManaColor::Green),
        "colorless" | "c" => Some(ManaColor::Colorless),
        "generic" | "any" => Some(ManaColor::Colorless),
        _ => None,
    }
}

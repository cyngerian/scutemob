use crate::rules::command::CastSpellData;
use crate::state::combat::AttackTarget;
use crate::state::types::{AdditionalCost, AltCostKind, FaceDownKind, TurnFaceUpMethod};
use crate::state::{ActivatedAbility, ActivationCost, CounterType, SacrificeFilter};
use crate::testing::script_schema::{
    ActionTarget, AttackerDeclaration, BlockerDeclaration, EnlistDeclaration, InitialState,
};
use crate::{
    all_cards, register_commander_zone_replacements, AbilityDefinition, CardDefinition,
    CardEffectTarget, CardId, CardRegistry, CardType, Color, Command, Condition, Cost,
    DeathTriggerFilter, Designations, ETBTriggerFilter, Effect, EffectAmount, GameState,
    GameStateBuilder, GameStateError, KeywordAbility, ManaAbility, ManaColor, ManaCost, ObjectSpec,
    PlayerId, PlayerTarget, Step, TargetController, TargetFilter, TargetRequirement,
    TimingRestriction, TriggerCondition, TriggerEvent, TriggeredAbilityDef, ZoneId,
};
use imbl::OrdMap;
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
// ── Determinism ───────────────────────────────────────────────────────────────
/// Iterate a script's `HashMap<String, T>` in key order.
///
/// `InitialState`'s zone and player maps are `std::collections::HashMap`s, so
/// their iteration order is seeded per map instance by `RandomState`. Two
/// deserializations of the *same* JSON in the *same* process can iterate in
/// different orders. Because [`build_initial_state`] assigns `ObjectId`s in
/// insertion order, iterating one of these maps directly makes the built
/// `GameState` nondeterministic: the same script yields different `ObjectId`
/// assignments, hence a different `public_state_hash`, run to run.
///
/// Every loop over a script-supplied map must go through this function
/// (SR-9b; `tests/scripts/harness_equivalence.rs` is the regression gate).
fn sorted_zone_entries<T>(map: &HashMap<String, T>) -> Vec<(&String, &T)> {
    let mut entries: Vec<(&String, &T)> = map.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    entries
}
// ── Public API ────────────────────────────────────────────────────────────────
/// Build a [`GameState`] from a [`GameScript`]'s initial state description.
///
/// Returns the state and a mapping from script player names → [`PlayerId`].
///
/// Player names are sorted alphabetically and assigned `PlayerId(1)`, `PlayerId(2)`, …
/// This is deterministic for a given set of player names.
///
/// # Relationship to the sealed `GameState` (SR-3)
///
/// This module is `pub` and is shared with the replay viewer, so it is compiled
/// into production builds — it therefore cannot use the `test-util` escape
/// hatches in [`crate::state::test_util`].
///
/// It does not need them. It lives *inside* the engine crate, so it patches the
/// `pub(crate)` fields directly for the few things [`GameStateBuilder`] cannot
/// express (life totals, mana pools, land plays). Crucially, the state is
/// finished before it escapes: this function hands back an owned [`GameState`]
/// by value and never lends `&mut GameState` to a caller. From outside the
/// engine, this is a documented constructor, not a mutation channel — which is
/// what keeps architecture invariant #3 intact.
///
/// # Completeness (Architecture Invariant 9 / SR-21)
///
/// This function is the **greppable opt-out**: it runs *no* completeness check,
/// so it will happily build a state out of an inert / partial / knowingly-wrong
/// `CardDefinition`. That is deliberate and load-bearing — engine test harnesses
/// (`tests/combat/combat_harness.rs`, the `script_replay` checker) and the
/// replay-viewer legitimately drive placeholder or not-yet-authored cards, and
/// they call *this* function precisely to say so. It is the replay-path analogue
/// of [`crate::rules::engine::start_game_allowing_incomplete`].
///
/// The replay-viewer opts out on purpose: ~20 *approved* golden scripts place a
/// card whose def is still marked non-`Complete` (the script exercises one
/// interaction while an unrelated clause of the def is unauthored), so gating it
/// would make the tool unable to view most of its own corpus.
///
/// A caller that runs a real game from a script and expects only `Complete` defs
/// should instead go through [`build_initial_state_checked`], which refuses a
/// known-but-non-`Complete` def exactly as `start_game` does. `grep` for
/// `build_initial_state(` (without `_checked`) to audit every opt-out site — the
/// point of SR-21 is that each such bypass is now a named, greppable choice, not
/// a silent one.
pub fn build_initial_state(init: &InitialState) -> (GameState, HashMap<String, PlayerId>) {
    // Sort player names deterministically.
    let mut names: Vec<String> = init.players.keys().cloned().collect();
    names.sort();
    let player_map: HashMap<String, PlayerId> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            // Checked conversion: `i` is a `usize` from `enumerate()`. Player
            // counts are always tiny, so this never fails in practice, but use
            // `try_into` rather than a bare `as u64` cast so an overflow would
            // panic loudly instead of silently wrapping.
            let index: u64 = i.try_into().expect("player index fits in u64");
            (name.clone(), PlayerId(index + 1))
        })
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

    // SR-9b: `init.turn_number` was declared by the schema and never read, so every
    // script ran on turn 1 regardless of what it said. `entered_turn` and every
    // "this turn" comparison read `turn.turn_number`, so a script that set up a
    // turn-5 board was silently playing a different game than it described.
    let mut builder = GameStateBuilder::new()
        .at_step(step)
        .active_player(active)
        .turn_number(init.turn_number.max(1))
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
    //
    // SR-9b: every zone map below is a `HashMap<String, _>`, whose iteration order
    // is randomized per instance by `RandomState`. Objects are assigned `ObjectId`s
    // in insertion order, so iterating these maps directly makes the resulting
    // `GameState` — and therefore its hash, and therefore anything ObjectId-ordered
    // downstream — differ between two builds of the *same* script in the *same*
    // process. Sort the keys. `sorted_zone_entries` exists so no future zone loop
    // forgets.
    for (ctrl_name, permanents) in sorted_zone_entries(&init.zones.battlefield) {
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
    for (owner_name, hand_cards) in sorted_zone_entries(&init.zones.hand) {
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
    for (owner_name, gy_cards) in sorted_zone_entries(&init.zones.graveyard) {
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
    for (owner_name, lib_cards) in sorted_zone_entries(&init.zones.library) {
        if let Some(&owner) = player_map.get(owner_name) {
            for card in lib_cards {
                builder = builder.object(make_spec(owner, &card.card, ZoneId::Library(owner)));
            }
        }
    }
    let mut state = builder.build().unwrap();
    // Patch life totals, mana pools, and land plays (can't do these via builder).
    for (name, pstate) in sorted_zone_entries(&init.players) {
        if let Some(&pid) = player_map.get(name) {
            if let Some(ps) = state.players.get_mut(&pid) {
                ps.life_total = pstate.life;
                for (color_str, amount) in sorted_zone_entries(&pstate.mana_pool) {
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
    for (name, pstate) in sorted_zone_entries(&init.players) {
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
/// Build a [`GameState`] from a script's initial state **and enforce Architecture
/// Invariant 9** (SR-21).
///
/// This is the checked entry for the script/replay path — the analogue of
/// [`crate::rules::engine::start_game`]. It builds the state exactly as
/// [`build_initial_state`] does, then runs the *same* completeness check
/// `start_game` runs (`check_all_defs_complete`): if any object references a
/// **known but non-`Complete`** `CardDefinition` (inert / partial / knowingly
/// wrong), it returns [`GameStateError::IncompleteCardsInGame`] instead of a
/// state.
///
/// Scope is deliberately narrow, matching `start_game` and `validate_deck`: a
/// `card_id` that is *absent* from the registry (a naked test object, an
/// un-authored name) is **not** an offender — it is the `UnknownCard` axis, and
/// gating it here would redden the hundreds of tests that build states against
/// an empty or partial registry. Only a def that exists *and* is marked fires.
///
/// This is the entry a caller uses when a script is expected to contain only
/// `Complete` defs — the replay-path analogue of [`crate::rules::engine::start_game`].
/// The replay-viewer deliberately does **not** use it (its corpus includes
/// approved fixtures with intentionally-incomplete defs; see
/// [`build_initial_state`]); the symbol exists so that any real-game consumer of
/// a script gets the same invariant-9 guarantee `start_game` gives, and so
/// `completeness_gate.rs` can prove the gate fires.
pub fn build_initial_state_checked(
    init: &InitialState,
) -> Result<(GameState, HashMap<String, PlayerId>), GameStateError> {
    let (state, player_map) = build_initial_state(init);
    crate::rules::engine::check_all_defs_complete(&state)?;
    Ok((state, player_map))
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
    // CR 701.43d / CR 508.1g: For `declare_attackers`, names of declared attackers the
    // player chooses to exert. Empty for no exert choices or all other action types.
    exert_names: &[String],
    // CR 118.9: For `cast_spell_pitch`, the name of the card in hand to exile as (part
    // of) the pitch alternative cost. `None` for pitch spells with no ExileFromHand
    // component, or for all other action types.
    pitch_exile_card_name: Option<&str>,
    // PB-EF12 (CR 605.3b / CR 106.1b): For `tap_for_mana` on an `any_color: true` mana
    // ability. One of "white"/"blue"/"black"/"red"/"green" (case-insensitive). `None` for
    // fixed-colour sources or all other action types.
    chosen_color_name: Option<&str>,
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
            let target_list = resolve_targets(targets, state, players)?;
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
            Some(Command::CastSpell(Box::new(CastSpellData {
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
                            // lki: vec![] — non-LKI path; engine fills in the value at the
                            // cost-payment site inside handle_cast_spell (CR 608.2b, PB-P).
                            costs.push(AdditionalCost::Sacrifice {
                                ids: vec![sac_id],
                                lki: vec![],
                            });
                        }
                    }
                    costs
                },
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })))
        }
        // CR 702.34a: Cast a spell with flashback from the player's graveyard.
        // The engine determines it's a flashback cast by checking the card's zone
        // (graveyard) and whether it has the Flashback keyword. No new Command variant
        // is needed — CastSpell handles flashback automatically when the source is
        // in the graveyard.
        "cast_spell_flashback" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.74a: Cast a spell with evoke from the player's hand.
        // The evoke cost (an alternative cost) is paid instead of the mana cost.
        "cast_spell_evoke" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.103a: Cast a spell with bestow from the player's hand.
        // The bestow cost (an alternative cost) is paid instead of the mana cost.
        "cast_spell_bestow" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.35a: Cast a madness card from exile by paying the madness cost.
        // The card is located in the caster's exile zone (put there by the discard
        // replacement effect). Madness is auto-detected from the card's zone + keyword.
        "cast_spell_madness" => {
            let card_id = find_in_exile(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.94a: Cast a miracle card from hand by paying the miracle cost.
        // The card is in hand (drawn this turn as first draw). A MiracleTrigger must
        // be on the stack (the player already chose to reveal via ChooseMiracle).
        "cast_spell_miracle" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.138a: Cast a spell with escape from the player's graveyard.
        // The escape cost (mana + exiling other cards) is paid instead of the mana cost.
        // The action uses the `escape` field: names of other cards to exile from graveyard.
        "cast_spell_escape" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            // Resolve escape exile card names to ObjectIds in the caster's graveyard.
            let exile_ids: Vec<crate::state::game_object::ObjectId> = escape_names
                .iter()
                .filter_map(|name| find_in_graveyard(state, player, name.as_str()))
                .collect();
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
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
            // PB-EF12 (CR 605.3b/106.1b): parse the script's chosen_color string, if any.
            // `None` when the script names no colour (fixed-colour source); the engine
            // rejects a mismatched combination (any_color-without-colour or
            // fixed-colour-with-colour) as an `InvalidCommand`.
            let chosen_color = chosen_color_name.and_then(|name| match name.to_lowercase().as_str() {
                "white" => Some(ManaColor::White),
                "blue" => Some(ManaColor::Blue),
                "black" => Some(ManaColor::Black),
                "red" => Some(ManaColor::Red),
                "green" => Some(ManaColor::Green),
                _ => None,
            });
            // Assume ability index 0 for basic mana abilities.
            Some(Command::TapForMana {
                player,
                source: source_id,
                ability_index: 0,
                chosen_color,
            })
        }
        "activate_ability" => {
            // PB-AC5: Some activated abilities function from zones other than the
            // battlefield (Channel/DiscardSelf from hand -- CR 702.34 -- and Transmute
            // from hand -- CR 702.53a; graveyard-activated abilities like Reassembling
            // Skeleton). Try battlefield first (the common case), then hand, then
            // graveyard, so a single harness action covers all activation zones.
            let name = card_name?;
            let source_id = find_on_battlefield(state, player, name)
                .or_else(|| find_in_hand(state, player, name))
                .or_else(|| find_in_graveyard(state, player, name))?;
            let target_list = resolve_targets(targets, state, players)?;
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
                // CR 700.2a/601.2b: mode indices chosen for a modal activated ability
                // (PB-EF7). Empty = non-modal, or auto-select mode 0.
                modes_chosen: modes_chosen.clone(),
            })
        }
        // CR 606: Activate a loyalty ability on a planeswalker.
        "activate_loyalty_ability" => {
            let source_id = find_on_battlefield(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
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
            let target_list = resolve_targets(targets, state, players)?;
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
            let target_list = resolve_targets(targets, state, players)?;
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
            // CR 701.43d / CR 508.1g: Resolve exert declarations to attacker ObjectIds.
            let exert_choices: Vec<crate::state::ObjectId> = exert_names
                .iter()
                .filter_map(|name| find_on_battlefield(state, player, name.as_str()))
                .collect();
            Some(Command::DeclareAttackers {
                player,
                attackers: atk_pairs,
                enlist_choices,
                exert_choices,
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
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
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
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.185a: Cast a spell by paying its warp cost. Tries hand (first cast),
        // then warped-in-exile (recast on a later turn), then graveyard (Timeline
        // Culler's `from_graveyard` permission) -- in that order.
        "cast_spell_warp" => {
            let name = card_name?;
            let card_id = find_in_hand(state, player, name)
                .or_else(|| find_warped_in_exile(state, player, name))
                .or_else(|| find_in_graveyard(state, player, name))?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Warp),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })))
        }
        // CR 118.9: Cast a spell by paying its pitch cost -- exile a card of the
        // required color from hand (pitch_exile_card_name) instead of paying the mana
        // cost, optionally combined with a life payment (Force of Will).
        "cast_spell_pitch" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            let pitch_id = pitch_exile_card_name.and_then(|name| find_in_hand(state, player, name));
            Some(Command::CastSpell(Box::new(CastSpellData {
                player,
                card: card_id,
                targets: target_list,
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Pitch),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: {
                    let mut costs = vec![];
                    if let Some(pid) = pitch_id {
                        costs.push(AdditionalCost::ExileFromHand { card: pid });
                    }
                    costs
                },
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })))
        }
        // CR 702.96a: Cast a spell with overload from the player's hand.
        // The overload cost (an alternative cost) is paid instead of the mana cost.
        // The spell has no targets -- it affects all valid objects.
        "cast_spell_overload" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            // CR 702.96b: Overloaded spells have no targets.
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.81a: Cast a spell with retrace from the player's graveyard.
        // The player discards a land card from hand as an additional cost.
        // The spell uses its normal mana cost (retrace is additional, not alternative).
        // After resolution the card returns to the graveyard normally (not exiled).
        "cast_spell_retrace" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            let land_name = discard_land_name?;
            let land_id = find_in_hand(state, player, land_name)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.133a: Cast a spell with jump-start from the player's graveyard.
        // The player pays the card's normal mana cost PLUS discards a card from hand.
        // Unlike retrace, the discarded card may be any card type (not just a land).
        // The card is exiled when it leaves the stack (resolves, countered, or fizzles).
        "cast_spell_jump_start" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            let discard_name = discard_card_name?;
            let discard_id = find_in_hand(state, player, discard_name)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.127a: Cast the aftermath half of a split card from the player's graveyard.
        // The aftermath half's mana cost is paid (alternative cost) and the card is exiled
        // when it leaves the stack. The card must have AbilityDefinition::Aftermath.
        "cast_spell_aftermath" => {
            let card_id = find_in_graveyard(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.160 / CR 718: Cast a spell using its prototype cost from the player's hand.
        // Prototype is NOT an alternative cost (CR 118.9 / 2022-10-14 ruling) — orthogonal
        // to alt_cost and can be combined with alternative costs like Flashback or Escape.
        "cast_spell_prototype" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.109a: Cast a spell with dash from the player's hand.
        // The dash cost (an alternative cost) is paid instead of the mana cost.
        "cast_spell_dash" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.152a: Cast a spell with blitz from the player's hand.
        // The blitz cost (an alternative cost) is paid instead of the mana cost.
        "cast_spell_blitz" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.176a: Cast a spell with impending from the player's hand.
        // The impending cost (an alternative cost) is paid instead of the mana cost.
        "cast_spell_impending" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.166a: Cast a spell with bargain from the player's hand, sacrificing
        // an artifact, enchantment, or token as the optional additional cost.
        // Bargain is an additional cost (CR 118.8), not an alternative cost -- the
        // spell's normal mana cost is still paid.
        "cast_spell_bargain" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            let bargain_sac_id =
                bargain_sacrifice_name.and_then(|name| find_on_battlefield(state, player, name));
            Some(Command::CastSpell(Box::new(CastSpellData {
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
                    .map(|id| {
                        vec![crate::state::types::AdditionalCost::Sacrifice {
                            ids: vec![id],
                            lki: vec![],
                        }]
                    })
                    .unwrap_or_default(),
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })))
        }
        // CR 701.59a: Cast a spell with collect evidence from the player's hand, exiling
        // cards from the caster's graveyard with total mana value >= N as an additional cost.
        // Collect evidence is an additional cost (CR 118.8), not an alternative cost -- the
        // spell's normal mana cost is still paid in full. Unlike Delve, the exiled cards do
        // NOT reduce the mana cost (CR 701.59a).
        "cast_spell_collect_evidence" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            // CR 701.59a: Resolve each evidence card name to an ObjectId in the caster's graveyard.
            let evidence_ids: Vec<crate::state::game_object::ObjectId> = collect_evidence_names
                .iter()
                .filter_map(|name| find_in_graveyard(state, player, name.as_str()))
                .collect();
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.119a: Cast a spell with emerge from the player's hand, sacrificing
        // a creature as part of the emerge alternative cost. The total mana cost is
        // reduced by the sacrificed creature's mana value.
        "cast_spell_emerge" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            let emerge_sac_name = emerge_sacrifice_name?;
            let emerge_sac_id = find_on_battlefield(state, player, emerge_sac_name)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
                additional_costs: vec![crate::state::types::AdditionalCost::Sacrifice {
                    ids: vec![emerge_sac_id],
                    lki: vec![],
                }],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })))
        }
        // CR 702.137a: Cast a spell with spectacle from the player's hand.
        // The spectacle cost (an alternative cost) is paid instead of the mana cost.
        // Precondition: an opponent of the casting player must have lost life this turn.
        "cast_spell_spectacle" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        "cast_spell_surge" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.153a: Cast a spell with casualty from the player's hand, optionally
        // sacrificing a creature with power >= N as the casualty additional cost.
        // Casualty is an additional cost (CR 118.8), not an alternative cost -- the
        // spell's normal mana cost is still paid.
        // If `casualty_sacrifice_name` is Some, the named creature is looked up on
        // the battlefield and sacrificed as the additional cost.
        "cast_spell_casualty" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            let casualty_sac_id =
                casualty_sacrifice_name.and_then(|name| find_on_battlefield(state, player, name));
            Some(Command::CastSpell(Box::new(CastSpellData {
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
                    .map(|id| {
                        vec![crate::state::types::AdditionalCost::Sacrifice {
                            ids: vec![id],
                            lki: vec![],
                        }]
                    })
                    .unwrap_or_default(),
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })))
        }
        // CR 702.132a: Cast a spell with assist from the player's hand. The assist player
        // pays some amount of the generic mana cost from their own mana pool. The caster
        // pays the remainder. Assist is not an alternative cost — the caster still pays
        // any colored mana pips plus whatever generic remains after assist.
        // `assist_player_name` identifies the assisting player; `assist_amount` is the
        // number of generic mana they pay.
        "cast_spell_assist" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            let assist_pid = assist_player_name.and_then(|name| players.get(name).copied());
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.56a: Cast a spell with the replicate additional cost paid N times.
        // `replicate_count` is the number of times the replicate cost is paid.
        // Each payment adds the replicate cost to the total mana cost.
        // If `replicate_count > 0`, a `ReplicateTrigger` is placed on the stack that
        // creates N copies of the original spell on resolution.
        "cast_spell_replicate" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.47a: Cast a spell with splice cards declared.
        // `splice_card_names` lists the names of cards in the caster's hand to splice
        // onto the spell. Each named card must have the Splice ability and be in hand.
        "cast_spell_splice" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
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
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.148a: Cast with Cleave — pay the cleave cost to remove bracketed text.
        "cast_spell_cleave" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
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
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.42a: Cast a modal spell with the entwine additional cost paid.
        // When entwine_paid = true, all modes of the spell are chosen and the entwine
        // cost is added to the total mana cost. The spell must have KeywordAbility::Entwine.
        "cast_spell_entwine" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.120a: Cast a modal spell with escalate additional cost paid.
        // `escalate_modes` is the number of extra modes beyond the first. The escalate
        // cost is multiplied by this count and added to the total mana cost.
        // Mode 0 plus `escalate_modes` additional modes execute at resolution.
        "cast_spell_escalate" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.82a: Cast with Devour -- sacrifice creatures as an ETB replacement effect.
        // `convoke_names` (reused parameter slot) lists the names of creatures on the
        // battlefield controlled by the caster to sacrifice. Empty list = no sacrifice (devour 0).
        // The sacrifice and counter placement happen at resolution (ETB replacement), not here.
        "cast_spell_devour" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            // Resolve each devour creature name to an ObjectId on the battlefield.
            let devour_ids: Vec<crate::state::game_object::ObjectId> = convoke_names
                .iter()
                .filter_map(|name| find_on_battlefield(state, player, name.as_str()))
                .collect();
            Some(Command::CastSpell(Box::new(CastSpellData {
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
                    vec![crate::state::types::AdditionalCost::Sacrifice {
                        ids: devour_ids,
                        lki: vec![],
                    }]
                },
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })))
        }
        // CR 700.2a / 601.2b: Cast a modal spell with explicit mode indices chosen.
        // `modes_chosen` specifies which mode indices (0-indexed) to execute at resolution.
        // For "choose one" spells: exactly one index (e.g., [0], [1], [2]).
        // For "choose two" spells: exactly two indices (e.g., [0, 2]).
        // For "choose up to N": between 1 and N indices.
        "cast_spell_modal" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.102a: Cast a fused split card from hand, paying the combined mana cost
        // of both halves (CR 702.102c). At resolution, the left half's effect executes
        // first, then the right half's (CR 702.102d). Card must be in the caster's hand
        // and must have KeywordAbility::Fuse and AbilityDefinition::Fuse.
        "cast_spell_fuse" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.157a: Cast a creature spell with the squad additional cost paid N times.
        // `squad_count` is the number of times the squad cost is paid (from the action).
        // Each payment adds the squad cost (from AbilityDefinition::Squad { cost }) to the
        // total mana cost. On ETB, a SquadTrigger creates N token copies of the creature.
        "cast_spell_squad" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 702.175a: Cast a creature spell with the offspring additional cost paid.
        // Sets `offspring_paid: true` on CastSpell. On ETB, an OffspringTrigger creates
        // 1 token copy of the creature except it's 1/1.
        "cast_spell_offspring" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
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
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
        }
        // CR 118.9 / Commander 2020 cycle: Cast a spell from hand without paying its mana cost,
        // conditional on controlling a commander on the battlefield.
        // 2020-04-17 ruling: any commander (any player's) satisfies the condition.
        // Action JSON fields:
        //   `card_name`: the card to cast from hand
        //   `targets`: optional target list
        "cast_spell_commander_free" => {
            let card_id = find_in_hand(state, player, card_name?)?;
            let target_list = resolve_targets(targets, state, players)?;
            Some(Command::CastSpell(Box::new(CastSpellData {
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
            })))
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
        // CR 701.28: Transform a double-faced permanent (Delver of Secrets, …).
        // `Command::Transform` has existed since M8; this arm did not, so `transform`
        // fell through to `_ => None` and every script using it silently no-op'd.
        "transform" => Some(Command::Transform {
            player,
            permanent: find_on_battlefield(state, player, card_name?)?,
        }),
        // ── Informational actions ─────────────────────────────────────────────
        //
        // These name a game event the engine performs on its own; there is no
        // `Command` to issue and none is missing. They are the script's prose, and
        // `tests/scripts/run_all_scripts.rs` allowlists them by name
        // (`ALLOWED_UNTRANSLATABLE_ACTIONS`) so that an action which is *not* on that
        // list can never no-op in silence again.
        //
        // CR 701.23a: the engine resolves `SearchLibrary` deterministically (minimum
        // `ObjectId` matching the filter). M10's interactive search will make this a
        // real `Command::SelectLibraryCard`.
        "search_library"
        // CR 510.1a-d: combat damage is assigned and dealt by the engine when the
        // combat damage step begins. A script records the assignment it expects; the
        // assertions after it observe the damage the engine actually dealt.
        | "assign_damage"
        // CR 601.2b / 700.2: a choice the engine makes deterministically at resolution
        // (which permanents to sacrifice to Annihilator, which mode to take). The
        // script documents the choice; the assertions check the engine made it.
        | "choose_option"
        // CR 701.17a: sacrifice as a cost or effect happens inside the ability that
        // demands it. There is no free-standing "sacrifice" command.
        | "sacrifice" => None,
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
/// PB-OS4b (CR 712.8d/e): builds the three runtime ability vectors
/// (`Characteristics.mana_abilities` / `activated_abilities` / `triggered_abilities`)
/// from a single face's `AbilityDefinition` list.
///
/// Extracted from `enrich_spec_from_def` (which calls this for the front face at
/// object construction) so `apply_face_change` can call it again for the *back*
/// face's abilities at the transform boundary (CR 712.8d/e: a transformed
/// permanent has only its currently-visible face's characteristics, including
/// abilities). Using the identical lowering logic for both faces means base ==
/// effective-face for these three vectors at all times, so every downstream
/// reader (direct-base or `calculate_characteristics`-based) is correct with no
/// per-reader auditing.
///
/// Preserves the mana/activated-ability disjointness (SR-34/SF-6): an ability
/// lowered into `mana_abilities` is excluded from `activated_abilities`.
pub(crate) fn build_face_ability_vectors(
    abilities: &[AbilityDefinition],
) -> (
    imbl::Vector<ManaAbility>,
    Vec<ActivatedAbility>,
    Vec<TriggeredAbilityDef>,
) {
    let mut mana_abilities: imbl::Vector<ManaAbility> = imbl::Vector::new();
    let mut activated_abilities: Vec<ActivatedAbility> = Vec::new();
    let mut triggered_abilities: Vec<TriggeredAbilityDef> = Vec::new();
    // SR-34 (CR 605.1a): convert mana-producing activated abilities into mana abilities.
    // CR 605.1a classifies an ability as a mana ability by what it *does* (no target,
    // could add mana, not a loyalty ability), not by what it costs — so this is no longer
    // gated on `matches!(cost, Cost::Tap)`. `mana_ability_lowering` (below) is the single
    // predicate deciding both this and the `activated_abilities` exclusion right below it,
    // so the two can never disagree (SR-34 §3 step 5 — they previously did, silently, for
    // `Effect::AddManaMatchingType`).
    for ability in abilities {
        if let AbilityDefinition::Activated {
            cost,
            effect,
            targets: ab_targets,
            activation_condition,
            ..
        } = ability
        {
            if let Some(ma) = mana_ability_lowering(ab_targets, cost, effect, activation_condition)
            {
                mana_abilities.push_back(ma);
            }
        }
    }
    // Populate non-mana activated abilities into characteristics.activated_abilities.
    // This is required so that Command::ActivateAbility can look up the ability by index.
    for ability in abilities {
        if let AbilityDefinition::Activated {
            cost,
            effect,
            timing_restriction,
            targets: ab_targets,
            activation_condition,
            activation_zone,
            once_per_turn,
            modes,
        } = ability
        {
            // Skip every ability the loop above registered as a ManaAbility (SR-34: no
            // longer just bare-Tap fixed/any-color/scaled/filter-choice). Including it here
            // too would shift ability_index for every non-mana activated ability that
            // follows it (SF-6).
            let is_tap_mana_ability =
                mana_ability_lowering(ab_targets, cost, effect, activation_condition).is_some();
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
                    // CR 700.2a: Propagate modal structure for modal activated abilities.
                    modes: modes.clone(),
                };
                activated_abilities.push(ab);
            }
        }
    }
    // CR 603.6c / CR 700.4: Convert "When ~ dies" card-definition triggers into
    // runtime TriggeredAbilityDef entries so check_triggers can dispatch them.
    // This covers self-referential dies triggers (e.g. Solemn Simulacrum).
    // CR 700.2b: For modal WhenDies triggers, use mode 0 as the bot fallback effect.
    for ability in abilities {
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
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
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
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenAttacks,
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
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                trigger_on: TriggerEvent::SelfAttacks,
                intervening_if: None,
                targets: targets.clone(),
                description: "Whenever ~ attacks (CR 508.3a)".to_string(),
                effect: Some(resolved_effect),
            });
        }
    }
    // CR 509.1: Convert "Whenever ~ blocks" card-definition triggers into runtime
    // TriggeredAbilityDef entries so check_triggers can dispatch them.
    // This covers self-referential block triggers (e.g. a creature with "Whenever ~ blocks").
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenBlocks,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                trigger_on: TriggerEvent::SelfBlocks,
                intervening_if: None,
                targets: targets.clone(),
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
    for ability in abilities {
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
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
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
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenDealtDamage,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                trigger_on: TriggerEvent::SelfIsDealtDamage,
                intervening_if: None,
                targets: targets.clone(),
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
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition:
                TriggerCondition::WheneverOpponentCastsSpell {
                    spell_type_filter,
                    noncreature_only,
                },
            effect,
            targets,
            ..
        } = ability
        {
            // Index-namespace fix (2026-07-09): carry spell_type_filter/noncreature_only
            // on the runtime TriggeredAbilityDef itself (reusing TargetFilter's
            // has_card_types/non_creature) instead of leaving the post-filter in
            // rules/abilities.rs to re-look the ability up via CardDefinition::abilities
            // (a different, non-dense index space -- see abilities.rs G-4 comment).
            let spell_filter = if spell_type_filter.is_some() || *noncreature_only {
                Some(TargetFilter {
                    has_card_types: spell_type_filter.clone().unwrap_or_default(),
                    non_creature: *noncreature_only,
                    ..Default::default()
                })
            } else {
                None
            };
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: spell_filter,
                trigger_on: TriggerEvent::OpponentCastsSpell,
                intervening_if: None,
                targets: targets.clone(),
                description: "Whenever an opponent casts a spell (CR 603.2)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 701.25d: Convert "Whenever you surveil" card-definition triggers into
    // runtime TriggeredAbilityDef entries so check_triggers can dispatch them
    // via Surveilled events.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouSurveil,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                trigger_on: TriggerEvent::ControllerSurveils,
                intervening_if: None,
                targets: targets.clone(),
                description: "Whenever you surveil (CR 701.25d)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 701.50b: Convert "Whenever this creature connives" card-definition triggers
    // into runtime TriggeredAbilityDef entries so check_triggers can dispatch them
    // via Connived events. Fires even if the creature left the battlefield
    // (Psychic Pickpocket ruling, 2022-04-29).
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenConnives,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                trigger_on: TriggerEvent::SourceConnives,
                intervening_if: None,
                targets: targets.clone(),
                description: "Whenever this creature connives (CR 701.50b)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 701.16a: Convert "Whenever you investigate" card-definition triggers into
    // runtime TriggeredAbilityDef entries so check_triggers can dispatch them
    // via Investigated events.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouInvestigate,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                trigger_on: TriggerEvent::ControllerInvestigates,
                intervening_if: None,
                targets: targets.clone(),
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
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition:
                TriggerCondition::WheneverYouCastSpell {
                    spell_type_filter,
                    noncreature_only,
                    spell_subtype_filter,
                    // The elided `chosen_subtype_filter` (CR 603.1 "of the chosen type",
                    // Vanquisher's Banner only) is a dynamic per-source condition (compared
                    // against the source's `chosen_creature_type` at trigger time) rather
                    // than a static TargetFilter predicate, so it cannot be folded into
                    // `triggering_creature_filter` below. It remains unenforced by this
                    // conversion — same as before this fix — and is out of scope here
                    // (see rules/abilities.rs G-4 comment; Vanquisher's Banner card-level
                    // fix is explicitly deferred).
                    ..
                },
            effect,
            targets,
            ..
        } = ability
        {
            // Index-namespace fix (2026-07-09): carry spell_type_filter/
            // noncreature_only/spell_subtype_filter on the runtime TriggeredAbilityDef
            // itself (reusing TargetFilter's has_card_types/non_creature/has_subtypes)
            // instead of leaving the post-filter in rules/abilities.rs to re-look the
            // ability up via CardDefinition::abilities (a different, non-dense index
            // space -- see abilities.rs G-4 comment; this was the root cause of the
            // Monastery Mentor / Leaf-Crowned Visionary filter bypass bug).
            let spell_filter = if spell_type_filter.is_some()
                || *noncreature_only
                || spell_subtype_filter.is_some()
            {
                Some(TargetFilter {
                    has_card_types: spell_type_filter.clone().unwrap_or_default(),
                    non_creature: *noncreature_only,
                    has_subtypes: spell_subtype_filter.clone().unwrap_or_default(),
                    ..Default::default()
                })
            } else {
                None
            };
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: spell_filter,
                trigger_on: TriggerEvent::ControllerCastsSpell,
                intervening_if: None,
                targets: targets.clone(),
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
    // PB-XS-E: `exclude_self` is now per-card. Set to `true` on cards whose oracle
    // text uses "another"/"other" (Alliance cards, Shadow Alley Denizen, Marwyn, etc.);
    // leave `false` on cards that say "this or another X" (Risen Reef, Ayara,
    // Bloomvine Regent, Satoru) and on simple "Whenever a creature enters under your
    // control" cards (Witty Roastmaster). For non-creature trigger sources the
    // value is irrelevant — the source can never be the entering creature — but
    // it should still match the oracle text for clarity.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition:
                TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter,
                    exclude_self,
                },
            effect,
            targets,
            ..
        } = ability
        {
            let etb_filter = ETBTriggerFilter {
                creature_only: true,
                controller_you: filter
                    .as_ref()
                    .is_some_and(|f| matches!(f.controller, TargetController::You)),
                exclude_self: *exclude_self,
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
                // Alliance filter is always creature (the creature_only flag handles it);
                // the explicit card_type_filter is not needed for this conversion.
                card_type_filter: None,
            };
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
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
                // PB-AC0 (CR 603.2 / CR 205.3 / CR 111.1): forward the full carddef
                // TargetFilter as triggering_creature_filter so has_subtype /
                // has_subtypes / is_nontoken / exclude_subtypes are honored on the
                // creature-ETB trigger path. The ETBTriggerFilter above only carries
                // creature_only / controller_you / exclude_self / color_filter /
                // card_type_filter. Mirrors the death-trigger conversion. CR 603.2.
                triggering_creature_filter: filter.clone(),
                targets: targets.clone(),
            });
        }
    }
    // PB-L (CR 207.2c / CR 603.2): Convert "Whenever a permanent enters the battlefield"
    // card-definition triggers into runtime TriggeredAbilityDef entries so check_triggers
    // can dispatch them via AnyPermanentEntersBattlefield events.
    //
    // This is the battlefield-side counterpart to collect_graveyard_carddef_triggers
    // (which handles the same TriggerCondition while the source is in the graveyard,
    // e.g. Bloodghast).
    //
    // Covers Landfall (ability word, CR 207.2c) and all other
    // "Whenever a [permanent type] [you control] enters" triggers:
    //   - Lotus Cobra, Evolution Sage, Jaddi Offshoot (Land + You)
    //   - Horn of Greed (Land, any controller)
    //   - Warstorm Surge, Puresteel Paladin (non-Land filters)
    //
    // Unlike Alliance (WheneverCreatureEntersBattlefield) which historically
    // hardcoded `exclude_self: true`, this variant has always defaulted to
    // `exclude_self: false` (a land you just played can satisfy your own
    // "whenever a land enters" trigger). PB-XS-E surfaces both as a per-card
    // field so cards can opt into self-exclusion when oracle text says "another"
    // for a filter that could match the source itself.
    //
    // Field mapping:
    //   - `creature_only: false` unless the filter specifies Creature
    //   - `card_type_filter: filter.has_card_type` (PB-L)
    //   - `exclude_self: trigger_condition.exclude_self` (PB-XS-E)
    //
    // Skips abilities with `trigger_zone: Some(TriggerZone::Graveyard)` — those are
    // handled by collect_graveyard_carddef_triggers at dispatch time and must NOT
    // be added to the battlefield spec (the spec lives on the battlefield object,
    // but these triggers fire only from the graveyard).
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition:
                TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter,
                    exclude_self,
                },
            effect,
            trigger_zone,
            targets,
            ..
        } = ability
        {
            // Graveyard-zone triggers (Bloodghast) are dispatched separately.
            if trigger_zone.is_some() {
                continue;
            }
            let (creature_only, card_type_filter, controller_you, color_filter) = match filter {
                Some(f) => {
                    let creature_only = matches!(f.has_card_type, Some(CardType::Creature));
                    // If the filter specifies a non-Creature card type, carry it in
                    // card_type_filter. For Creature, the creature_only flag handles it.
                    let card_type_filter = match f.has_card_type {
                        Some(CardType::Creature) => None,
                        other => other,
                    };
                    let controller_you = matches!(f.controller, TargetController::You);
                    let color_filter = f.colors.as_ref().and_then(|colors| {
                        if colors.len() == 1 {
                            colors.iter().next().copied()
                        } else {
                            None
                        }
                    });
                    (
                        creature_only,
                        card_type_filter,
                        controller_you,
                        color_filter,
                    )
                }
                None => (false, None, false, None),
            };
            let etb_filter = ETBTriggerFilter {
                creature_only,
                controller_you,
                // PB-XS-E (CR 109.1 / 603.2): per-card "another" gate. The default
                // (`false`) keeps Landfall-style triggers firing for the entering
                // land you just played. Setting `true` matches oracle text using
                // "another [permanent type]" — used when the filter could match
                // the trigger source itself (e.g. a creature with a layer-effect
                // type addition matching its own filter).
                exclude_self: *exclude_self,
                color_filter,
                card_type_filter,
            };
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
                // card_definition::Condition ↔ runtime InterveningIf conversion is
                // deferred (same rationale as Alliance). None is safe for all
                // known simple Landfall cards; compound-conditional cases
                // (Moraug, Omnath Locus of Creation) remain TODO-blocked.
                intervening_if: None,
                description: "Whenever a permanent enters the battlefield (CR 207.2c / CR 603.2)"
                    .to_string(),
                effect: Some(effect.clone()),
                etb_filter: Some(etb_filter),
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 702.140d: Convert "Whenever this creature mutates" card-definition triggers
    // into runtime TriggeredAbilityDef entries so check_triggers can dispatch them
    // via CreatureMutated events. Only fires on the merged permanent itself (CR 729.2c).
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenMutates,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                trigger_on: TriggerEvent::SelfMutates,
                intervening_if: None,
                targets: targets.clone(),
                description: "Whenever this creature mutates (CR 702.140d)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // "Whenever this permanent becomes tapped" — fires on any tap event (mana, combat,
    // opponent effects). Used by City of Brass. Maps to TriggerEvent::SelfBecomesTapped
    // which is already dispatched from GameEvent::PermanentTapped in check_triggers.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenSelfBecomesTapped,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                trigger_on: TriggerEvent::SelfBecomesTapped,
                intervening_if: None,
                targets: targets.clone(),
                description: "Whenever this permanent becomes tapped".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 502.3 / 603.2e (PB-AC1): Convert "Whenever a permanent becomes untapped" card-definition
    // triggers into runtime TriggeredAbilityDef entries so check_triggers can dispatch them via
    // AnyPermanentUntaps events. The optional filter is forwarded to `triggering_creature_filter`
    // (a legacy field name reused for any triggering object, not just creatures) and applied at
    // trigger-collection time in `collect_triggers_for_event`.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverPermanentUntaps { filter },
            effect,
            once_per_turn,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: *once_per_turn,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: filter.clone(),
                trigger_on: TriggerEvent::AnyPermanentUntaps,
                intervening_if: None,
                targets: targets.clone(),
                description: "Whenever a permanent becomes untapped (CR 502.3/603.2e)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 122.6 / 122.7 (PB-AC1): Convert "When/Whenever counter(s) are put on [this
    // permanent] / [a permanent you control]" card-definition triggers into runtime
    // TriggeredAbilityDef entries so check_triggers can dispatch them via CounterPlaced
    // events. `counter` and `on_self` are forwarded directly to the runtime `counter_filter` /
    // `counter_on_self` fields; `filter` (for the on_self:false "a creature you control" case)
    // is forwarded to `triggering_creature_filter`.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition:
                TriggerCondition::WhenCounterPlaced {
                    counter,
                    filter,
                    on_self,
                },
            effect,
            once_per_turn,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: counter.clone(),
                counter_on_self: *on_self,
                once_per_turn: *once_per_turn,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: filter.clone(),
                trigger_on: TriggerEvent::CounterPlaced,
                intervening_if: None,
                targets: targets.clone(),
                description:
                    "Whenever one or more counters are put on a permanent (CR 122.6/122.7)"
                        .to_string(),
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
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition:
                TriggerCondition::WheneverCreatureDies {
                    controller,
                    exclude_self,
                    nontoken_only,
                    filter,
                },
            effect,
            once_per_turn,
            targets,
            ..
        } = ability
        {
            let death_filter = DeathTriggerFilter {
                controller_you: matches!(controller, Some(TargetController::You)),
                controller_opponent: matches!(controller, Some(TargetController::Opponent)),
                exclude_self: *exclude_self,
                nontoken_only: *nontoken_only,
            };
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                // CR 603.2c/603.2h (PB-AC1): forward "triggers only once each turn" (Morbid
                // Opportunist) from the card def.
                once_per_turn: *once_per_turn,
                trigger_on: TriggerEvent::AnyCreatureDies,
                intervening_if: None,
                description: "Whenever a creature dies (CR 603.10a)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: Some(death_filter),
                combat_damage_filter: None,
                // PB-N: forward the DSL subtype/color filter to the runtime field (CR 603.10a)
                triggering_creature_filter: filter.clone(),
                targets: targets.clone(),
            });
        }
    }
    // CR 508.1m / CR 603.2: Convert "Whenever a creature you control attacks" card-definition
    // triggers into runtime TriggeredAbilityDef entries so check_triggers can dispatch them
    // via AnyCreatureYouControlAttacks events.
    //
    // Controller filtering is applied at trigger-collection time by checking that the attacking
    // creature's controller matches the trigger source's controller.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            // PB-N: WheneverCreatureYouControlAttacks changed from unit to struct variant
            trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks { filter },
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::AnyCreatureYouControlAttacks,
                intervening_if: None,
                description: "Whenever a creature you control attacks (CR 508.1m)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                // PB-N: forward the DSL subtype/color filter to the runtime field (CR 508.1m)
                triggering_creature_filter: filter.clone(),
                // PB-EF3 A1: forward the DSL declared targets (CR 601.2c) instead of
                // dropping them — see abilities.rs A2 for the matching fallback fix.
                targets: targets.clone(),
            });
        }
    }
    // CR 510.3a / CR 603.2: Convert "Whenever a creature you control deals combat damage to a
    // player" card-definition triggers into runtime TriggeredAbilityDef entries so check_triggers
    // can dispatch them via AnyCreatureYouControlDealsCombatDamageToPlayer events.
    //
    // Controller filtering is applied at trigger-collection time by checking that the source
    // creature's controller matches the trigger source's controller.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition:
                TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer { filter },
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer,
                intervening_if: None,
                description:
                    "Whenever a creature you control deals combat damage to a player (CR 510.3a)"
                        .to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: filter.clone(),
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 510.3a / CR 603.2c: Convert "Whenever one or more creatures you control deal combat
    // damage to a player" batch trigger into runtime TriggeredAbilityDef entries.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition:
                TriggerCondition::WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer { filter },
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::AnyCreatureYouControlBatchCombatDamage,
                intervening_if: None,
                description:
                    "Whenever one or more creatures you control deal combat damage to a player (CR 510.3a)"
                        .to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: filter.clone(),
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 510.3a: Convert "Whenever equipped creature deals combat damage to a player" triggers.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::EquippedCreatureDealsCombatDamageToPlayer,
                intervening_if: None,
                description:
                    "Whenever equipped creature deals combat damage to a player (CR 510.3a)"
                        .to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 510.3a: Convert "Whenever equipped creature deals combat damage" (any recipient).
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamage,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::EquippedCreatureDealsCombatDamage,
                intervening_if: None,
                description: "Whenever equipped creature deals combat damage (CR 510.3a)"
                    .to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 510.3a: Convert "Whenever enchanted creature deals damage to a player" triggers.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer { .. },
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::EnchantedCreatureDealsDamageToPlayer,
                intervening_if: None,
                description: "Whenever enchanted creature deals damage to a player (CR 510.3a)"
                    .to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 510.3a / CR 603.2: Convert "Whenever a creature deals combat damage to one of your
    // opponents" (Edric) triggers into runtime TriggeredAbilityDef entries.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenAnyCreatureDealsCombatDamageToOpponent,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::AnyCreatureDealsCombatDamageToOpponent,
                intervening_if: None,
                description:
                    "Whenever a creature deals combat damage to one of your opponents (CR 510.3a)"
                        .to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 701.9a: Convert "Whenever you discard a card" triggers.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouDiscard,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::ControllerDiscards,
                intervening_if: None,
                description: "Whenever you discard a card (CR 701.9a)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 701.9a: Convert "Whenever an opponent discards a card" triggers.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverOpponentDiscards,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::OpponentDiscards,
                intervening_if: None,
                description: "Whenever an opponent discards a card (CR 701.9a)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 305.1: Convert "Whenever an opponent plays a land" triggers.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverOpponentPlaysLand,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::OpponentPlaysLand,
                intervening_if: None,
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
                description: "Whenever an opponent plays a land (CR 305.1)".to_string(),
                effect: Some(effect.clone()),
            });
        }
    }
    // CR 701.21a: Convert "Whenever you sacrifice a permanent" triggers.
    // player_filter=None → ControllerSacrifices (fires only when controller sacrifices).
    // player_filter=Some(Any) → ControllerSacrifices (any player; filtered at dispatch time).
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouSacrifice { .. },
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::ControllerSacrifices,
                intervening_if: None,
                description: "Whenever you sacrifice a permanent (CR 701.21a)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 508.1: Convert "Whenever you attack" triggers.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouAttack,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::ControllerAttacks,
                intervening_if: None,
                description: "Whenever you attack (CR 508.1)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 603.10a: Convert "When ~ leaves the battlefield" triggers.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenLeavesBattlefield,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::SelfLeavesBattlefield,
                intervening_if: None,
                description: "When ~ leaves the battlefield (CR 603.10a)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 603.2: Convert "Whenever you draw a card" triggers (WheneverYouDrawACard).
    // Maps to ControllerDrawsCard event — dispatched via CardDrawn.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouDrawACard,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::ControllerDrawsCard,
                intervening_if: None,
                description: "Whenever you draw a card (CR 603.2)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 603.2: Convert "Whenever a player draws a card" triggers (WheneverPlayerDrawsCard).
    // player_filter=None → AnyPlayerDrawsCard, Some(Opponent) → OpponentDrawsCard,
    // Some(You) → ControllerDrawsCard.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverPlayerDrawsCard { player_filter },
            effect,
            targets,
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
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on,
                intervening_if: None,
                description: desc.to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 603.2 / CR 118.4: Convert "Whenever you gain life" triggers.
    // Maps to ControllerGainsLife — dispatched via LifeGained.
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverYouGainLife,
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::ControllerGainsLife,
                intervening_if: None,
                description: "Whenever you gain life (CR 603.2)".to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    // CR 603.2: WhenYouCastThisSpell is dispatched directly from the SpellCast arm
    // in check_triggers using the CardDef ability_index. No TriggeredAbilityDef needed.
    // PB-AC6 / CR 601.2c / 602.2b / 603.2: Convert "Whenever [this permanent / a <filter>
    // you control] becomes the target of a spell [or ability] [an opponent controls]"
    // card-definition triggers into runtime TriggeredAbilityDef entries. Only `trigger_on`
    // carries the scope/by_opponent/include_abilities params -- the inline dispatch in
    // `check_triggers`'s `GameEvent::PermanentTargeted` arm reads them directly from the
    // `TriggerEvent::PermanentBecomesTarget` variant (NOT via the generic equality-based
    // `collect_triggers_for_event`, since each card's params differ).
    for ability in abilities {
        if let AbilityDefinition::Triggered {
            trigger_condition:
                TriggerCondition::WhenBecomesTarget {
                    scope,
                    by_opponent,
                    include_abilities,
                },
            effect,
            targets,
            ..
        } = ability
        {
            triggered_abilities.push(TriggeredAbilityDef {
                counter_filter: None,
                counter_on_self: false,
                once_per_turn: false,
                trigger_on: TriggerEvent::PermanentBecomesTarget {
                    scope: scope.clone(),
                    by_opponent: *by_opponent,
                    include_abilities: *include_abilities,
                },
                intervening_if: None,
                description: "Whenever ~ becomes the target of a spell/ability (CR 601.2c/602.2b)"
                    .to_string(),
                effect: Some(effect.clone()),
                etb_filter: None,
                death_filter: None,
                combat_damage_filter: None,
                triggering_creature_filter: None,
                targets: targets.clone(),
            });
        }
    }
    (mana_abilities, activated_abilities, triggered_abilities)
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
    // Face-aware ability gathering (CR 712.8d/e, PB-OS4b): the mana-ability lowering,
    // non-mana activated-ability lowering, and triggered-ability lowering below are
    // extracted into `build_face_ability_vectors` so `apply_face_change` can rebuild the
    // same three vectors from the *back* face's abilities at the transform boundary,
    // using byte-identical logic. This call lowers the *front* face (`def.abilities`).
    // Front-face-only concerns that are NOT part of these three vectors (keyword
    // markers, the Reconfigure/Outlast/Champion/Soulbond/Cipher/Gift keyword tags) stay
    // inline below unchanged. The triggered-ability vector is applied further down, at
    // the point the old inline loops used to begin, to preserve the original code's
    // application order exactly.
    let (front_mana_abilities, front_activated_abilities, front_triggered_abilities) =
        build_face_ability_vectors(&def.abilities);
    for ma in front_mana_abilities {
        spec = spec.with_mana_ability(ma);
    }
    for ab in front_activated_abilities {
        spec = spec.with_activated_ability(ab);
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
                modes: None,
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
                modes: None,
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
                    exert: false,
                    life_cost: 0,
                    sacrifice_exclude_self: false,
                    exile_self_from_hand: false,
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
                modes: None,
            };
            spec = spec.with_activated_ability(ab);
        }
    }
    // Apply the front-face triggered-ability vector computed above by
    // `build_face_ability_vectors` (extracted from what used to be inlined here).
    for t in front_triggered_abilities {
        spec = spec.with_triggered_ability(t);
    }
    spec
}
// ── Private helpers ───────────────────────────────────────────────────────────
/// Resolve a script action's target list to engine [`Target`](crate::state::Target)s.
///
/// Returns `None` if **any** target fails to resolve.
///
/// SR-9b: this used to `filter_map`, silently *dropping* an unresolvable target
/// and handing the engine a shorter list. A `cast_spell` naming one permanent
/// that is not on the battlefield therefore became a `CastSpell` with an empty
/// `targets` vec — a targeted spell cast with no target at all (CR 601.2c) — and
/// the script went green. Dropping a target is never what a script meant; a name
/// that does not resolve is a broken script, and the action must not translate.
fn resolve_targets(
    targets: &[ActionTarget],
    state: &GameState,
    players: &HashMap<String, PlayerId>,
) -> Option<Vec<crate::state::Target>> {
    targets
        .iter()
        .map(|t| {
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
/// CR 702.185a/b: Find a named warped card in exile owned by the given player.
///
/// Warped cards are in ZoneId::Exile with `Designations::WARPED` set. Unlike general
/// exile (which is a shared zone), warped cards are filtered by owner.
fn find_warped_in_exile(
    state: &GameState,
    player: PlayerId,
    name: &str,
) -> Option<crate::state::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name
            && obj.zone == ZoneId::Exile
            && obj.designations.contains(Designations::WARPED)
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
/// The cost components of a mana ability's activation cost, as `handle_tap_for_mana`
/// (`rules/mana.rs`) can pay them. Mirrors `ManaAbility`'s cost fields.
struct ManaAbilityCost {
    requires_tap: bool,
    sacrifice_self: bool,
    mana_cost: Option<ManaCost>,
    life_cost: u32,
    exile_self_from_hand: bool,
}
/// SR-34 (CR 605.1a): decompose a `Cost` into the components a mana ability can pay
/// through `Command::TapForMana { player, source, ability_index, chosen_color }`. Returns `None` if
/// any component needs a channel `TapForMana` does not have — a caller-supplied
/// `ObjectId` (discard a specific card, sacrifice *another* permanent, remove a counter)
/// — in which case the ability is not lowerable and stays a stack-using activated
/// ability. Accepts `Cost::Tap`, `Cost::Mana(_)`, `Cost::PayLife(_)`, `Cost::SacrificeSelf`
/// (already honoured by `handle_tap_for_mana` via `ManaAbility::sacrifice_self` —
/// e.g. Treasure tokens), and any `Cost::Sequence` composed only of those.
///
/// **SR-34 review Finding 1 (CR 601.2h)**: a second `Cost::Mana` component in the same
/// sequence declines rather than overwriting the first — `Cost::Sequence([Mana({1}),
/// Tap, Mana({1})])` must not silently lower to `mana_cost = {1}`. "Partial payments are
/// not allowed"; a cost this function cannot represent correctly must not lower at all.
/// No card in the corpus has two `Cost::Mana` components today (verified by grep across
/// every `Cost::Sequence` def); this is a latent-defect guard, not a live fix.
///
/// **SR-34 review Finding 2 (CR 106.12)**: a cost with no `Cost::Tap` component declines
/// to lower, even though `ManaAbility::requires_tap: false` exists and is otherwise
/// payable. Before this fix, `Cost::Mana`-only (Elvish/Simian Spirit Guide) and
/// `Cost::SacrificeSelf`-only (Food Chain) costs lowered into free, repeatable,
/// stackless mana abilities via a path (`handle_tap_for_mana` skips the tapped-status
/// check and has no exhaustion mechanism when `requires_tap` is false) that has zero
/// test coverage and was explicitly flagged as unproven by
/// `memory/primitives/sr34-affected-defs.md`. All three affected defs are non-`Complete`
/// today, so this closes the seam at zero cost rather than proving it correct. To
/// re-open it: add a test lowering a synthetic `Cost::Mana`-only or
/// `Cost::SacrificeSelf`-only def and asserting `TapForMana` pays/produces correctly
/// with no tapped-status check — see `sr34-affected-defs.md` and
/// `memory/card-authoring/sr34-engine-findings-2026-07-17.md` (Finding 2 discussion).
fn mana_ability_cost_components(cost: &Cost) -> Option<ManaAbilityCost> {
    fn walk(cost: &Cost, acc: &mut ManaAbilityCost) -> bool {
        match cost {
            Cost::Tap => {
                acc.requires_tap = true;
                true
            }
            Cost::Mana(m) => {
                if acc.mana_cost.is_some() {
                    // A second Cost::Mana component: this function cannot merge two
                    // mana costs without risking a silent partial payment. Decline.
                    false
                } else {
                    acc.mana_cost = Some(m.clone());
                    true
                }
            }
            Cost::PayLife(n) => {
                acc.life_cost += n;
                true
            }
            Cost::SacrificeSelf => {
                acc.sacrifice_self = true;
                true
            }
            Cost::Sequence(costs) => costs.iter().all(|c| walk(c, acc)),
            // PB-EF8 (CR 605.1a): a from-hand exile-self cost IS lowerable — the source
            // is `self`, identified by `Command::TapForMana`'s ObjectId, exactly like
            // the already-accepted `SacrificeSelf`. No caller-supplied payload needed.
            Cost::ExileSelfFromHand => {
                acc.exile_self_from_hand = true;
                true
            }
            // Not lowerable: needs a caller-supplied ObjectId (Sacrifice(filter),
            // RemoveCounter, DiscardCard) or is a hand/self-exile alt-cost shape
            // (DiscardSelf, ExileSelf, ExileFromHand, Forage, Exert) that
            // `Command::TapForMana` has no payload for.
            Cost::Sacrifice(_)
            | Cost::RemoveCounter { .. }
            | Cost::DiscardCard
            | Cost::DiscardSelf
            | Cost::Forage
            | Cost::ExileSelf
            | Cost::ExileFromHand { .. }
            | Cost::Exert => false,
        }
    }
    let mut acc = ManaAbilityCost {
        requires_tap: false,
        sacrifice_self: false,
        mana_cost: None,
        life_cost: 0,
        exile_self_from_hand: false,
    };
    if !walk(cost, &mut acc) {
        return None;
    }
    // PB-EF8 (CR 605.1a / CR 400.7): relax the no-tap guard for an
    // `exile_self_from_hand` cost. The SR-34 guard below declines a *no-tap* cost
    // because such a cost has no exhaustion mechanism and would register a free,
    // repeatable, stackless mana ability (Elvish/Simian Spirit Guide were the exact
    // seam it closed). An `exile_self_from_hand` cost is inherently one-shot and
    // self-consuming — the source leaves hand and becomes a dead ObjectId (CR 400.7),
    // so it cannot be activated again — so the seam does not apply. This relaxation is
    // scoped to *only* this flag: a `Cost::Mana`-only or `SacrificeSelf`-only no-tap
    // cost (Food Chain) is still declined below.
    if !acc.requires_tap && !acc.exile_self_from_hand {
        // SR-34 review Finding 2: decline to lower a no-tap cost. See the doc comment
        // above for the seam this leaves closed and how to re-open it deliberately.
        return None;
    }
    Some(acc)
}
/// **The single predicate deciding mana-ability lowering (SR-34, CR 605.1a).** Used both
/// to build the `ManaAbility` (`enrich_spec_from_def`'s mana-ability loop) and to decide
/// the `activated_abilities` exclusion right after it — the same call, so the two lists
/// can never disagree (SR-34 §3 step 5; they previously did for `AddManaMatchingType`,
/// silently, because no card exercised the gap).
///
/// CR 605.1a: an activated ability is a mana ability iff it has no target, could add mana
/// when it resolves, and is not a loyalty ability (loyalty abilities never reach this
/// function — see the `AbilityDefinition::Activated` match in `enrich_spec_from_def`).
/// `targets.is_empty()` is the first criterion; `mana_ability_cost_components` bounds which
/// cost shapes this engine can pay without a stack (see its doc); `try_as_tap_mana_ability`
/// recognises the mana-producing effect shapes.
///
/// **SR-36: `AddManaScaled` is no longer excluded from any cost shape.** SF-8 gave
/// `handle_tap_for_mana` an `AddManaScaled` branch (it resolves `ManaAbility::scaled_amount`
/// via `resolve_amount`), so the ex-Finding-A exclusion that used to keep every
/// non-bare-`Cost::Tap` `AddManaScaled` ability on the stack (Cabal Coffers, Cabal
/// Stronghold, Crypt of Agadeem) is gone — `try_as_tap_mana_ability`'s `AddManaScaled` arm
/// now carries the real amount for any cost shape `mana_ability_cost_components` accepts.
fn mana_ability_lowering(
    targets: &[TargetRequirement],
    cost: &Cost,
    effect: &Effect,
    activation_condition: &Option<Condition>,
) -> Option<ManaAbility> {
    // CR 605.1a: "it doesn't require a target".
    if !targets.is_empty() {
        return None;
    }
    let components = mana_ability_cost_components(cost)?;
    let mut ma = try_as_tap_mana_ability(effect)?;
    ma.requires_tap = components.requires_tap;
    ma.sacrifice_self = components.sacrifice_self;
    ma.mana_cost = components.mana_cost;
    ma.life_cost = components.life_cost;
    ma.exile_self_from_hand = components.exile_self_from_hand;
    // SR-37 / SF-10 (CR 605.1a + CR 602.5b): carry an "activate only if ..." restriction
    // into the ManaAbility. CR 605.1a keeps a conditioned ability a mana ability (so it is
    // still lowered), but the condition must be enforced at activation — `handle_tap_for_mana`
    // checks it. Before SR-37 the lowering loop's `..` dropped this field, so Tainted Field's
    // coloured arms produced {W}/{B} with no Swamp controlled.
    ma.activation_condition = activation_condition.clone().map(Box::new);
    Some(ma)
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
            produces: imbl::OrdMap::new(),
            requires_tap: true,
            sacrifice_self: false,
            any_color: true,
            damage_to_controller: 0,
            ..Default::default()
        });
    }
    // Filter land pattern: {Hybrid},{T}: AddManaFilterChoice { color_a, color_b }
    // Produces 1 of color_a + 1 of color_b (middle option of 3 choices).
    if let Effect::AddManaFilterChoice {
        color_a, color_b, ..
    } = effect
    {
        let mut produces = imbl::OrdMap::new();
        produces.insert(*color_a, 1u32);
        *produces.entry(*color_b).or_insert(0) += 1;
        return Some(ManaAbility {
            produces,
            requires_tap: true,
            sacrifice_self: false,
            any_color: false,
            damage_to_controller: 0,
            ..Default::default()
        });
    }
    // Scaled mana: {T}: AddManaScaled { color, count }
    // `produces={color: 1}` is kept as the colour-channel marker SR-33's
    // `every_complete_land_registers_each_printed_tap_mana_color` gate reads;
    // `scaled_amount` carries the real `EffectAmount` for `handle_tap_for_mana` to
    // resolve at activation (SR-36 — see the doc comment on `ManaAbility::scaled_amount`).
    //
    // SR-36 / SR-38 (SG-2): the stackless `TapForMana` path always pays the activating
    // player, so a scaled mana ability that adds mana to a player OTHER than its controller
    // (`player` here is not `PlayerTarget::Controller`) cannot be lowered — declining
    // (returning `None`) leaves it on the stack, where `Effect::AddManaScaled`'s
    // stack-resolution arm handles an arbitrary `PlayerTarget` correctly.
    //
    // This is a compromise, not a free "slow path". CR 605.3b: a mana ability doesn't use
    // the stack, so an ability kept there hands opponents a priority window it should never
    // grant — precisely the defect SR-33 fixed by rewriting 88 dual lands, and why Cabal
    // Coffers was `Partial` rather than `Complete`. It is accepted here only because it is
    // unreachable: every mana source in the corpus adds to its controller, so this branch is
    // dead for real cards (verified via `all_cards()`). It guards solely against a future def
    // that would otherwise silently register a free, stackless mana ability paying the wrong
    // player. Pinned by `opponent_scaled_mana_stays_a_stack_ability` (SR-38).
    if let Effect::AddManaScaled {
        player,
        color,
        count,
    } = effect
    {
        if !matches!(player, PlayerTarget::Controller) {
            return None;
        }
        let mut p = imbl::OrdMap::new();
        p.insert(*color, 1u32);
        return Some(ManaAbility {
            produces: p,
            requires_tap: true,
            sacrifice_self: false,
            any_color: false,
            damage_to_controller: 0,
            scaled_amount: Some(Box::new(count.clone())),
            ..Default::default()
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
                    ..
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
        ..Default::default()
    })
}
/// Convert a card-definition [`Cost`] into an [`ActivationCost`] for object characteristics.
///
/// Handles `Tap`, `Mana`, `Sacrifice`, `DiscardCard`, `PayLife` (SR-36) and `Sequence`
/// (recursively). See `flatten_cost_into` for the components still without an
/// `ActivationCost` representation — each is ignored here and named there.
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
            // PB-EF1 (CR 109.1): preserve the "another" restriction. `SacrificeFilter`
            // has no self-exclusion, so the bit is carried on `ActivationCost` and
            // enforced in `handle_activate_ability` (abilities.rs).
            ac.sacrifice_exclude_self = filter.exclude_self;
        }
        Cost::Sequence(costs) => costs.iter().for_each(|c| flatten_cost_into(c, ac)),
        Cost::DiscardCard => ac.discard_card = true,
        Cost::DiscardSelf => ac.discard_self = true,
        Cost::Forage => ac.forage = true,
        Cost::ExileSelf => ac.exile_self = true,
        // SR-36: `+=`, not `=` — a `Cost::Sequence` may hold more than one `PayLife`
        // component and this walk is recursive (CR 118.3 / 119.4).
        Cost::PayLife(n) => ac.life_cost += *n,
        Cost::ExileFromHand { .. } => {} // pitch is a spell alt cost, not an activation cost
        Cost::ExileSelfFromHand => ac.exile_self_from_hand = true,
        Cost::Exert => ac.exert = true,
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

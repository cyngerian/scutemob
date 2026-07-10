//! Regression tests for the `ability_index` namespace desync bug (2026-07-09).
//!
//! `PendingTrigger.ability_index` is a *dense* index into a runtime, layer-resolved
//! `characteristics.triggered_abilities` list (set by `collect_triggers_for_event` in
//! `rules/abilities.rs` via `resolved_chars.triggered_abilities.iter().enumerate()`).
//! A previously-buggy post-filter for `ControllerCastsSpell`/`OpponentCastsSpell`
//! triggers (spell_type_filter / noncreature_only / spell_subtype_filter enforcement)
//! instead looked the ability up via `def.abilities.get(t.ability_index)` — an index
//! into the *raw* `CardDefinition::abilities` Vec, which also contains
//! Keyword/Static/Activated abilities. These two index spaces only coincide when a
//! card's Triggered ability happens to sit at the same position in both, which is
//! false for any multi-ability card whose Triggered ability isn't first. On such
//! cards the lookup landed on the wrong (non-Triggered) `AbilityDefinition`, fell
//! through the match's `_ => true` catch-all, and the filter was silently skipped.
//!
//! Monastery Mentor (`Keyword(Prowess)` at `abilities[0]`, `Triggered { noncreature_only:
//! true }` at `abilities[1]`, but the sole runtime triggered ability at dense index 0)
//! is a real, previously-shipped card broken by this bug: it created a Monk token on
//! every spell cast, including creature spells.
//!
//! See also `pb_ac7_card_integration.rs::test_leaf_crowned_visionary_full_integration`
//! for the `spell_subtype_filter` case (Static ability at `abilities[0]` desyncing a
//! `Triggered` ability at `abilities[1]`).

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    enrich_spec_from_def, process_command, CardDefinition, CardId, CardRegistry, CardType, Command,
    GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step, TriggerEvent,
    ZoneId,
};
use std::collections::HashMap;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn defs_of(def: &CardDefinition) -> HashMap<String, CardDefinition> {
    let mut m = HashMap::new();
    m.insert(def.name.clone(), def.clone());
    m
}

fn card_spec(
    player: PlayerId,
    name: &str,
    card_id: &str,
    zone: ZoneId,
    def: &CardDefinition,
) -> ObjectSpec {
    enrich_spec_from_def(ObjectSpec::card(player, name), &defs_of(def))
        .with_card_id(CardId(card_id.to_string()))
        .in_zone(zone)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn monk_token_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects()
        .values()
        .filter(|o| {
            o.zone == ZoneId::Battlefield
                && o.controller == player
                && o.is_token
                && o.characteristics.name == "Monk"
        })
        .count()
}

fn cast_spell(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<(GameState, Vec<GameEvent>), mtg_engine::GameStateError> {
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player,
            card,
            targets: vec![],
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
        })),
    )
}

fn pass_all(state: GameState, players: &[PlayerId]) -> GameState {
    let mut current = state;
    for &pl in players {
        current = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e))
            .0;
    }
    current
}

fn resolve_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(guard < 100, "resolve_stack exceeded safety guard");
        state = pass_all(state, players);
    }
    state
}

// ── 1. Direct unit test: dense index resolves to the correct AbilityDefinition ────

/// CR 603.1 / CR 603.2: verifies the exact mechanism that was broken. Monastery
/// Mentor has `Keyword(Prowess)` at `CardDefinition::abilities[0]` and
/// `Triggered { noncreature_only: true, .. }` at `abilities[1]`, but its ONLY
/// runtime triggered ability is built at dense index 0 (`enrich_spec_from_def`
/// only pushes `Triggered` entries onto `triggered_abilities`, skipping Keyword).
/// `PendingTrigger.ability_index` for this trigger is therefore `0`, and it MUST
/// resolve (via `characteristics.triggered_abilities[0]`) to the noncreature-only
/// token-creation trigger, not to Prowess (which has no runtime TriggeredAbilityDef
/// representation at all).
#[test]
fn test_ability_index_resolves_dense_runtime_list_not_carddef_index() {
    let def = mtg_engine::cards::defs::monastery_mentor::card();
    // Sanity: confirm the CardDef-index mismatch this bug depended on.
    assert!(
        matches!(
            def.abilities[0],
            mtg_engine::AbilityDefinition::Keyword(mtg_engine::KeywordAbility::Prowess)
        ),
        "monastery_mentor.abilities[0] must be Keyword(Prowess) for this test to be meaningful"
    );
    assert!(
        matches!(
            def.abilities[1],
            mtg_engine::AbilityDefinition::Triggered { .. }
        ),
        "monastery_mentor.abilities[1] must be the Triggered token-creation ability"
    );

    let spec = card_spec(
        p(1),
        "Monastery Mentor",
        "monastery-mentor",
        ZoneId::Battlefield,
        &def,
    );
    // The runtime triggered_abilities list is dense: exactly one entry (Prowess is a
    // Keyword, not a Triggered ability, and contributes no runtime TriggeredAbilityDef).
    assert_eq!(
        spec.triggered_abilities.len(),
        1,
        "only the Triggered ability produces a runtime TriggeredAbilityDef entry"
    );
    let trigger_def = &spec.triggered_abilities[0];
    // Dense index 0 must resolve to the ControllerCastsSpell trigger with its
    // noncreature_only filter intact -- NOT silently drop the filter.
    assert_eq!(trigger_def.trigger_on, TriggerEvent::ControllerCastsSpell);
    let filter = trigger_def
        .triggering_creature_filter
        .as_ref()
        .expect("noncreature_only must be carried on triggering_creature_filter");
    assert!(
        filter.non_creature,
        "TargetFilter.non_creature must mirror CardDef noncreature_only: true"
    );
}

// ── 2. Card integration: Monastery Mentor noncreature_only enforcement ────────────

/// CR 702.108a / CR 603.1: "Whenever you cast a noncreature spell, create a 1/1
/// white Monk creature token with prowess." Casting a CREATURE spell must NOT
/// create a Monk token (this is exactly the case the ability_index bug broke --
/// Monastery Mentor shipped creating a token on every spell, including creature
/// spells, because the noncreature_only filter was silently skipped).
#[test]
fn test_monastery_mentor_noncreature_only_creature_spell_no_token() {
    let def = mtg_engine::cards::defs::monastery_mentor::card();
    let registry = CardRegistry::new(vec![def.clone()]);
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card_spec(
            p1,
            "Monastery Mentor",
            "monastery-mentor",
            ZoneId::Battlefield,
            &def,
        ))
        .object(
            ObjectSpec::card(p1, "Test Creature Spell")
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Hand(p1)),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 10;
    state.turn_mut().priority_holder = Some(p1);

    let creature_spell_id = find_object(&state, "Test Creature Spell");
    let monks_before = monk_token_count(&state, p1);

    let (state, _) = cast_spell(state, p1, creature_spell_id).unwrap();
    let state = resolve_stack(state, &[p1, p2]);

    assert_eq!(
        monk_token_count(&state, p1),
        monks_before,
        "casting a CREATURE spell must NOT create a Monk token (noncreature_only filter)"
    );
}

/// CR 702.108a / CR 603.1: the positive case -- casting a NONCREATURE spell must
/// create exactly one Monk token.
#[test]
fn test_monastery_mentor_noncreature_only_noncreature_spell_creates_token() {
    let def = mtg_engine::cards::defs::monastery_mentor::card();
    let registry = CardRegistry::new(vec![def.clone()]);
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card_spec(
            p1,
            "Monastery Mentor",
            "monastery-mentor",
            ZoneId::Battlefield,
            &def,
        ))
        .object(
            ObjectSpec::card(p1, "Test Noncreature Spell")
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Hand(p1)),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 10;
    state.turn_mut().priority_holder = Some(p1);

    let noncreature_spell_id = find_object(&state, "Test Noncreature Spell");
    let monks_before = monk_token_count(&state, p1);

    let (state, _) = cast_spell(state, p1, noncreature_spell_id).unwrap();
    let state = resolve_stack(state, &[p1, p2]);

    assert_eq!(
        monk_token_count(&state, p1),
        monks_before + 1,
        "casting a NONCREATURE spell must create exactly one Monk token"
    );
}

// ── 3. TargetFilter reuse sanity: empty filter never restricts ────────────────────

/// Sanity check for the TargetFilter-reuse mechanism itself: a `WheneverYouCastSpell`
/// condition with none of `spell_type_filter`/`noncreature_only`/`spell_subtype_filter`
/// set must convert to `triggering_creature_filter: None` (no spurious restriction
/// introduced for the common unfiltered case, e.g. Inexorable Tide-style "whenever
/// you cast a spell" triggers).
#[test]
fn test_spell_cast_conversion_no_filter_when_carddef_unrestricted() {
    let def = CardDefinition {
        card_id: mtg_engine::cards::defs::monastery_mentor::card()
            .card_id
            .clone(),
        name: "Unrestricted Caster".to_string(),
        abilities: vec![mtg_engine::AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: mtg_engine::TriggerCondition::WheneverYouCastSpell {
                during_opponent_turn: false,
                spell_type_filter: None,
                noncreature_only: false,
                chosen_subtype_filter: false,
                spell_subtype_filter: None,
            },
            effect: mtg_engine::Effect::DrawCards {
                player: mtg_engine::PlayerTarget::Controller,
                count: mtg_engine::EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    };
    let spec = card_spec(
        p(1),
        "Unrestricted Caster",
        "unrestricted-caster",
        ZoneId::Battlefield,
        &def,
    );
    assert_eq!(spec.triggered_abilities.len(), 1);
    assert_eq!(
        spec.triggered_abilities[0].triggering_creature_filter, None,
        "no filter fields set on the CardDef -> no runtime TargetFilter should be attached"
    );
}

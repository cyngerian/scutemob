//! PB-EF2 (scutemob-102): `TokenSpec.recipient: PlayerTarget` — player-scoped recipient
//! for `Effect::CreateToken` / `Effect::CreateTokenAndAttachSource` (CR 111.1 / CR 608.2h).
//!
//! Fixes EF-W-MISS-1 (HIGH): `Effect::CreateToken` always minted for `ctx.controller` (the
//! resolving effect's controller), so Swan Song ("**Its controller** creates a 2/2 Bird")
//! handed the Bird to Swan Song's *caster* instead of the countered spell's controller.
//!
//! `TokenSpec` gains `recipient: PlayerTarget` (`#[serde(default)]`, default `Controller` —
//! byte-identical to every existing site). `PlayerTarget` gains two variants:
//! `ControllerOfCounteredSpell` (captured into `EffectContext::countered_spell_controller` by
//! `Effect::CounterSpell` the moment a valid target position is found, BEFORE the
//! `cant_be_countered` check — An Offer ruling 2022-04-29: an uncounterable-but-legal target's
//! controller still creates the tokens) and `ControllerOfTriggeringObject` (resolved from
//! `EffectContext::triggering_creature_id`'s controller, falling back to `triggering_player`,
//! then `controller`).
//!
//! Each test below is a decoy test in the SR-36 sense: the setup is arranged so that if
//! `recipient` were ignored (the pre-fix behaviour — always `ctx.controller`) the assertion
//! would fail. All scenarios are probed by EXECUTION: real spells are cast and resolved
//! through `process_command`, or the production `execute_effect` entry point is called
//! directly (never a stub) and real board state is read back afterward.
//!
//! HASH_SCHEMA_VERSION 44 -> 45 (`TokenSpec.recipient` field; `PlayerTarget` discriminants
//! 8/9). PROTOCOL_VERSION 6 -> 7 (same two types are in the `Command`/`GameEvent` wire
//! closure via `Effect::CreateToken`).

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::cards::card_definition::{CardDefinition, EffectAmount, PlayerTarget, TokenSpec};
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::rules::replacement::register_permanent_replacement_abilities;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, treasure_token_spec,
    AbilityDefinition, CardId, CardRegistry, CardType, Command, Effect, GameEvent, GameState,
    GameStateBuilder, ManaAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step,
    Target, TypeLine, ZoneId, HASH_SCHEMA_VERSION,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_obj_in_zone(state: &GameState, name: &str, zone: ZoneId) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in zone {:?}", name, zone))
}

fn count_on_battlefield_for(state: &GameState, name: &str, controller: PlayerId) -> usize {
    state
        .objects()
        .values()
        .filter(|o| {
            o.zone == ZoneId::Battlefield
                && o.characteristics.name == name
                && o.controller == controller
        })
        .count()
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        events.extend(ev);
    }
    (current, events)
}

/// A harmless untargeted noncreature instant used as the "victim" spell to counter. A real
/// card risks collateral behaviour (extra targets, replacement effects); this one has none,
/// so any recipient mismatch can only come from the field under test.
fn victim_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("ef2-victim-instant".to_string()),
        name: "EF2 Victim Instant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A vanilla creature spell — NOT a legal Swan Song target (oracle: instant/sorcery/
/// enchantment only). Used to pin the target-type restriction on the now-Complete def.
fn victim_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("ef2-victim-creature".to_string()),
        name: "EF2 Victim Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        oracle_text: "".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

fn all_defs() -> HashMap<String, CardDefinition> {
    let mut m: HashMap<String, CardDefinition> = all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect();
    let victim = victim_instant_def();
    m.insert(victim.name.clone(), victim);
    let victim_creature = victim_creature_def();
    m.insert(victim_creature.name.clone(), victim_creature);
    m
}

fn registry() -> Arc<CardRegistry> {
    let mut cards = all_cards();
    cards.push(victim_instant_def());
    cards.push(victim_creature_def());
    CardRegistry::new(cards)
}

fn id_for(name: &str) -> CardId {
    match name {
        "EF2 Victim Instant" => CardId("ef2-victim-instant".to_string()),
        "EF2 Victim Creature" => CardId("ef2-victim-creature".to_string()),
        _ => card_name_to_id(name),
    }
}

fn enrich(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(id_for(name)),
        defs,
    )
}

/// Cast `name` from `caster`'s hand, optionally targeting `target`. Resets priority to the
/// caster before casting (mirrors the in-response-to-a-spell-on-the-stack pattern).
fn cast(
    state: GameState,
    caster: PlayerId,
    name: &str,
    mana: &[(ManaColor, u32)],
    target: Option<ObjectId>,
) -> GameState {
    let card = find_obj_in_zone(&state, name, ZoneId::Hand(caster));
    let mut state = state;
    {
        let pool = &mut state.players_mut().get_mut(&caster).unwrap().mana_pool;
        for &(color, n) in mana {
            pool.add(color, n);
        }
    }
    state.turn_mut().priority_holder = Some(caster);
    let targets = target.map(|t| vec![Target::Object(t)]).unwrap_or_default();
    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: caster,
            card,
            targets,
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
    .unwrap_or_else(|e| panic!("CastSpell {} failed: {:?}", name, e));
    state
}

/// Cast `victim` from p1's hand, pass priority once (giving p2 a window), then cast
/// `counterer` from p2's hand targeting the victim on the stack. Returns the state with
/// both spells cast but not yet resolved (caller drains the stack via `pass_all`).
fn counter_scenario(p1: PlayerId, p2: PlayerId, counterer: &str) -> GameState {
    let defs = all_defs();
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(enrich(p1, "EF2 Victim Instant", ZoneId::Hand(p1), &defs))
        .object(enrich(p2, counterer, ZoneId::Hand(p2), &defs))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let state = cast(
        state,
        p1,
        "EF2 Victim Instant",
        &[(ManaColor::Colorless, 1)],
        None,
    );
    let (state, _) = pass_all(state, &[p1]);
    let victim_on_stack = find_obj_in_zone(&state, "EF2 Victim Instant", ZoneId::Stack);
    cast(
        state,
        p2,
        counterer,
        &[(ManaColor::Blue, 1)],
        Some(victim_on_stack),
    )
}

// ── Live HASH/PROTOCOL sentinel ──────────────────────────────────────────────

#[test]
fn test_pb_ef2_hash_schema_version_live_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 54u8,
        "PB-EF2 added TokenSpec.recipient + two PlayerTarget variants (HASH 44->45). Update \
         this sentinel and the state/hash.rs history block together; the authoritative check \
         is the SR-17 machine gate in tests/core/hash_schema.rs."
    );
}

// ── 1/2: Swan Song happy path + decoy ────────────────────────────────────────

/// CR 701.5g / EF-W-MISS-1: Swan Song's oracle text is "Counter target instant, sorcery, or
/// enchantment spell. **Its controller** creates a 2/2 blue Bird creature token with flying."
/// p1 casts a spell; p2 casts Swan Song targeting it. The Bird must be controlled by p1 (the
/// countered spell's controller), not p2 (Swan Song's caster).
#[test]
fn test_swan_song_bird_goes_to_countered_spells_controller() {
    let p1 = p(1);
    let p2 = p(2);
    let state = counter_scenario(p1, p2, "Swan Song");
    let (state, events) = pass_all(state, &[p1, p2, p1, p2]);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { player, .. } if *player == p1)),
        "CR 701.5: Swan Song should counter p1's victim spell"
    );
    assert_eq!(
        count_on_battlefield_for(&state, "Bird", p1),
        1,
        "EF-W-MISS-1: the 2/2 Bird must be controlled by p1 (the countered spell's \
         controller), not Swan Song's caster p2."
    );
}

/// Decoy for the same scenario: the field-under-test decoy must fail if `recipient` were
/// ignored (pre-fix behaviour always mints for `ctx.controller`, i.e. p2).
#[test]
fn test_swan_song_decoy_bird_is_not_controlled_by_swan_songs_caster() {
    let p1 = p(1);
    let p2 = p(2);
    let state = counter_scenario(p1, p2, "Swan Song");
    let (state, _) = pass_all(state, &[p1, p2, p1, p2]);

    assert_eq!(
        count_on_battlefield_for(&state, "Bird", p2),
        0,
        "Decoy: a pre-fix engine (recipient defaults to ctx.controller unconditionally) would \
         give Swan Song's CASTER (p2) the Bird. It must go to the countered spell's controller \
         (p1) instead."
    );
}

/// CR 115.1 / target restriction: Swan Song's oracle counters only "instant, sorcery, or
/// enchantment" spells. Now that swan_song ships Complete, its `TargetSpellWithFilter`
/// (has_card_types = [Instant, Sorcery, Enchantment]) must REJECT a creature spell as an
/// illegal target — bare `TargetSpell` (pre-review) would have let it counter anything.
#[test]
fn test_swan_song_cannot_target_a_creature_spell() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = all_defs();
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(enrich(p1, "EF2 Victim Creature", ZoneId::Hand(p1), &defs))
        .object(enrich(p2, "Swan Song", ZoneId::Hand(p2), &defs))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 casts the creature spell; it goes on the stack.
    let state = cast(
        state,
        p1,
        "EF2 Victim Creature",
        &[(ManaColor::Colorless, 1)],
        None,
    );
    let (mut state, _) = pass_all(state, &[p1]);
    let creature_on_stack = find_obj_in_zone(&state, "EF2 Victim Creature", ZoneId::Stack);

    // p2 attempts Swan Song targeting the creature spell — must be rejected.
    let swan = find_obj_in_zone(&state, "Swan Song", ZoneId::Hand(p2));
    state
        .players_mut()
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn_mut().priority_holder = Some(p2);
    let result = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p2,
            card: swan,
            targets: vec![Target::Object(creature_on_stack)],
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
    );
    assert!(
        result.is_err(),
        "Swan Song must not be castable targeting a creature spell (oracle: instant, sorcery, \
         or enchantment only). Bare TargetSpell would have wrongly permitted it."
    );
}

// ── 3: An Offer You Can't Refuse ─────────────────────────────────────────────

/// CR 701.5g: "Counter target noncreature spell. Its controller creates two Treasure
/// tokens." Same recipient fix, different token (also verifies each Treasure keeps its
/// mana ability — CR 111.10a — after the per-recipient rewrite of the CreateToken executor).
#[test]
fn test_an_offer_you_cant_refuse_creates_two_treasures_for_countered_spells_controller() {
    let p1 = p(1);
    let p2 = p(2);
    let state = counter_scenario(p1, p2, "An Offer You Can't Refuse");
    let (state, events) = pass_all(state, &[p1, p2, p1, p2]);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { player, .. } if *player == p1)),
        "CR 701.5: An Offer You Can't Refuse should counter p1's victim spell"
    );
    let treasures: Vec<ObjectId> = state
        .objects()
        .iter()
        .filter(|(_, o)| {
            o.zone == ZoneId::Battlefield
                && o.characteristics.name == "Treasure"
                && o.controller == p1
        })
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(
        treasures.len(),
        2,
        "'Its controller creates two Treasure tokens' — both must be controlled by p1 (the \
         countered spell's controller), not p2 (An Offer's caster)."
    );
    assert_eq!(
        count_on_battlefield_for(&state, "Treasure", p2),
        0,
        "Decoy: no Treasure should be controlled by the caster p2."
    );
    for id in treasures {
        let obj = state.objects().get(&id).unwrap();
        assert!(
            !obj.characteristics.mana_abilities.is_empty(),
            "CR 111.10a: each Treasure token must carry its sacrifice-for-mana ability"
        );
    }
}

// ── 4: default recipient is unchanged ────────────────────────────────────────

/// `TokenSpec::default().recipient == PlayerTarget::Controller`, and every existing
/// `..Default::default()`/helper (`treasure_token_spec`, etc.) site inherits it unmodified.
/// A plain `CreateToken` with the default recipient must still mint for `ctx.controller`.
#[test]
fn test_create_token_default_recipient_is_still_controller() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .build()
        .unwrap();
    let mut ctx = EffectContext::new(p1, ObjectId(0), vec![]);
    let _ = execute_effect(
        &mut state,
        &Effect::CreateToken {
            spec: treasure_token_spec(1),
        },
        &mut ctx,
    );

    assert_eq!(
        count_on_battlefield_for(&state, "Treasure", p1),
        1,
        "TokenSpec::default().recipient is PlayerTarget::Controller — the token must still \
         go to ctx.controller when recipient is not overridden."
    );
    assert_eq!(
        count_on_battlefield_for(&state, "Treasure", p2),
        0,
        "Decoy: nothing should go to the non-controller player."
    );
}

// ── 5: ControllerOfTriggeringObject resolves ─────────────────────────────────

/// `PlayerTarget::ControllerOfTriggeringObject` resolves from
/// `EffectContext::triggering_creature_id`'s controller. Direct unit test on the effect
/// executor: `ctx.controller` is p1 (the ability's controller) but `triggering_creature_id`
/// names a creature controlled by p2 — the token must go to p2, not p1.
#[test]
fn test_controller_of_triggering_object_resolves_to_triggering_creatures_controller() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p2, "Attacker", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();
    let attacker_id = find_obj(&state, "Attacker");

    let mut ctx = EffectContext {
        triggering_creature_id: Some(attacker_id),
        ..EffectContext::new(p1, ObjectId(0), vec![])
    };
    let _ = execute_effect(
        &mut state,
        &Effect::CreateToken {
            spec: TokenSpec {
                recipient: PlayerTarget::ControllerOfTriggeringObject,
                ..treasure_token_spec(1)
            },
        },
        &mut ctx,
    );

    assert_eq!(
        count_on_battlefield_for(&state, "Treasure", p2),
        1,
        "ControllerOfTriggeringObject must resolve to the triggering creature's controller \
         (p2), read from EffectContext::triggering_creature_id."
    );
    assert_eq!(
        count_on_battlefield_for(&state, "Treasure", p1),
        0,
        "Decoy: ctx.controller is p1, but the token must NOT go to the ability's controller \
         when ControllerOfTriggeringObject names a different player."
    );
}

// ── 6: token doubling keys off recipient, not controller ────────────────────

fn doubling_season_scenario(
    doubler_controller: Option<PlayerId>,
) -> (GameState, PlayerId, PlayerId) {
    let p1 = p(1); // recipient (stand-in for the countered spell's controller)
    let p2 = p(2); // ctx.controller (stand-in for Swan Song's caster)
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry());
    if let Some(owner) = doubler_controller {
        let mut ds_spec =
            ObjectSpec::artifact(owner, "Doubling Season").in_zone(ZoneId::Battlefield);
        ds_spec.card_id = Some(CardId("doubling-season".to_string()));
        builder = builder.object(ds_spec);
    }
    let mut state = builder.build().unwrap();
    if let Some(owner) = doubler_controller {
        let ds_id = find_obj(&state, "Doubling Season");
        let card_id = CardId("doubling-season".to_string());
        let registry = state.card_registry().clone();
        register_permanent_replacement_abilities(
            &mut state,
            ds_id,
            owner,
            Some(&card_id),
            &registry,
        );
    }
    (state, p1, p2)
}

/// Doubling Season is controlled by the RECIPIENT (p1), not by `ctx.controller` (p2, the
/// Swan-Song-caster stand-in). CR 111.1 / CR 614.1: token-creation replacements key off the
/// player who would create the tokens — the recipient — so p1 must get 2 tokens even though
/// p2 (whose Doubling Season it is NOT) is the resolving effect's controller.
#[test]
fn test_doubling_season_on_recipients_side_doubles_recipients_tokens() {
    let (mut state, p1, p2) = doubling_season_scenario(Some(p(1)));
    let mut ctx = EffectContext {
        countered_spell_controller: Some(p1),
        ..EffectContext::new(p2, ObjectId(0), vec![])
    };
    let _ = execute_effect(
        &mut state,
        &Effect::CreateToken {
            spec: TokenSpec {
                recipient: PlayerTarget::ControllerOfCounteredSpell,
                ..treasure_token_spec(1)
            },
        },
        &mut ctx,
    );

    assert_eq!(
        count_on_battlefield_for(&state, "Treasure", p1),
        2,
        "CR 111.1: Doubling Season on the recipient's (p1's) side must double the 1 Treasure \
         into 2, even though the resolving effect's controller (p2) is a different player."
    );
}

/// Reverse decoy: Doubling Season is controlled by `ctx.controller` (p2, the caster
/// stand-in), NOT by the recipient (p1). If doubling were (incorrectly) keyed to
/// `ctx.controller` instead of the recipient, p1 would get 2 tokens here too. It must not —
/// p1 gets exactly 1, proving `apply_token_creation_replacement` is called per-recipient.
#[test]
fn test_doubling_season_on_callers_side_does_not_double_recipients_tokens() {
    let (mut state, p1, p2) = doubling_season_scenario(Some(p(2)));
    let mut ctx = EffectContext {
        countered_spell_controller: Some(p1),
        ..EffectContext::new(p2, ObjectId(0), vec![])
    };
    let _ = execute_effect(
        &mut state,
        &Effect::CreateToken {
            spec: TokenSpec {
                recipient: PlayerTarget::ControllerOfCounteredSpell,
                ..treasure_token_spec(1)
            },
        },
        &mut ctx,
    );

    assert_eq!(
        count_on_battlefield_for(&state, "Treasure", p1),
        1,
        "Decoy: Doubling Season belongs to p2 (ctx.controller), not the recipient p1. \
         Doubling must NOT apply — p1 gets exactly 1 Treasure, proving the replacement is \
         keyed to the recipient, not ctx.controller."
    );
}

// Sanity: `ManaAbility::treasure()` stays reachable from this file's imports (used only
// implicitly via `treasure_token_spec`, but kept explicit here so a future edit that drops
// the import is caught by a compile error rather than an unused-import warning downgrade).
#[allow(dead_code)]
fn _uses_mana_ability_treasure() -> ManaAbility {
    ManaAbility::treasure()
}

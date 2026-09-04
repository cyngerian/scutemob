//! PB-DX18 (`OOS-M11-5`) — CR 601.2c: a spell that requires no targets cannot be given
//! any.
//!
//! `validate_targets_inner`'s empty-requirements arm used to pass every declared target
//! through with `None` as its requirement ("existence-only validation"), record them on
//! the `StackObject`, and — since PB-DX48 — hand them to
//! `rules::events::push_target_announcement`, which derives
//! `GameEvent::PermanentTargeted` and dispatches CR 702.21a **Ward** from it.
//!
//! So the defect was not a wrong label. **Ward fired off a spell that does not target.**
//!
//! Observed, not inferred (M11-local S5, `scutemob-167`): casting Accorder's Shield —
//! `{0}`, `Completeness::Complete`, deck-legal, whose SPELL declares no
//! `TargetRequirement` at all — with `params.targets = [Target::Player(2)]` returned
//! HTTP 200 and recorded the bogus player target.

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, CardId, CardRegistry, Command, GameEvent,
    GameState, GameStateBuilder, GameStateError, KeywordAbility, ManaColor, ObjectId, ObjectSpec,
    PlayerId, Step, Target, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn defs_by_name() -> std::collections::HashMap<String, mtg_engine::CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn find(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object {name:?} not found"))
}

fn cast(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player,
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
}

/// Two seats, `p1` holding Accorder's Shield (the seed's own observed subject) and `p2`
/// controlling a Ward creature so the CR 702.21a consequence is reachable.
fn shield_state() -> GameState {
    let registry = CardRegistry::new(all_cards());
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p(1), "Accorder's Shield")
                .with_card_id(CardId("accorders-shield".to_string()))
                .in_zone(ZoneId::Hand(p(1))),
        )
        .object(
            ObjectSpec::creature(p(2), "Warded Bear", 2, 2)
                .in_zone(ZoneId::Battlefield)
                .with_keyword(KeywordAbility::Ward(2)),
        )
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(1));
    state
}

#[test]
/// CR 601.2c — the seed's own observation, as a probe: Accorder's Shield with a declared
/// player target is REFUSED.
///
/// The card is `Completeness::Complete` and deck-legal, and its **spell** declares no
/// `TargetRequirement`. (The word "target" does appear in its stored oracle text, in the
/// Equip reminder — that belongs to the equip ACTIVATED ability, not to the spell. The
/// seed row's original wording got this wrong and was corrected by the S5 re-review; it
/// is restated here so the next reader does not re-derive the same confusion.)
fn t1_accorders_shield_refuses_a_declared_player_target() {
    let state = shield_state();
    let shield = find(&state, "Accorder's Shield");
    let err = cast(state, p(1), shield, vec![Target::Player(p(2))])
        .expect_err("CR 601.2c: a spell that requires no targets cannot be given one");
    match err {
        GameStateError::InvalidTarget(m) => assert!(
            m.contains("requires none") && m.contains("601.2c"),
            "the refusal must name the rule; got {m:?}"
        ),
        other => panic!("expected InvalidTarget, got {other:?}"),
    }
}

#[test]
/// CONTROL — the same cast with NO targets still works. The gate must refuse more than
/// HEAD did and nothing else.
fn t2_accorders_shield_still_casts_with_no_targets() {
    let state = shield_state();
    let shield = find(&state, "Accorder's Shield");
    let (state, events) =
        cast(state, p(1), shield, vec![]).expect("a targetless cast is still legal");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "the spell must reach the stack; events: {events:?}"
    );
    assert_eq!(state.stack_objects().len(), 1);
}

#[test]
/// **CR 702.21a — Ward provably no longer fires off a spell that does not target.**
///
/// This is `OOS-M11-5`'s real consequence and the reason it is a wrong-game-state bug
/// rather than a wrong-label one. Before PB-DX18 this cast was ACCEPTED, the bogus target
/// was recorded on the `StackObject`, and `casting.rs`'s `push_target_announcement` then
/// emitted `GameEvent::PermanentTargeted` for it — which since PB-DX48 is exactly what
/// dispatches Ward.
///
/// Asserted on BOTH observables the seed names, and by COUNT rather than by presence
/// (PB-DX48's rule: a `>= 1` assertion passes on the broken design too).
fn t3_ward_does_not_fire_off_a_spell_that_does_not_target() {
    let state = shield_state();
    let shield = find(&state, "Accorder's Shield");
    let bear = find(&state, "Warded Bear");

    let refused = cast(state, p(1), shield, vec![Target::Object(bear)]);
    let events = match refused {
        Err(GameStateError::InvalidTarget(_)) => Vec::new(),
        Err(other) => panic!("expected InvalidTarget, got {other:?}"),
        Ok((state, events)) => {
            // If the cast is ever accepted again, the assertions below are what report
            // the Ward consequence rather than merely the label.
            assert_eq!(
                state
                    .stack_objects()
                    .iter()
                    .filter(|so| {
                        matches!(
                            so.kind,
                            mtg_engine::StackObjectKind::TriggeredAbility { .. }
                        )
                    })
                    .count(),
                0,
                "a spell that does not target must put no ward trigger on the stack"
            );
            events
        }
    };
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, GameEvent::PermanentTargeted { .. }))
            .count(),
        0,
        "CR 601.2c / 702.21a: a spell that requires no targets must emit NO \
         PermanentTargeted, so Ward cannot be dispatched from it"
    );
}

#[test]
/// CONTROL — the case the deleted comment named. An Aura DOES still cast with its target.
///
/// The empty-requirements arm's in-source justification was *"used by auras/bestow which
/// validate via a separate enchant path"*. PB-DX20 made that false: an Aura's CR 303.4a
/// requirement is synthesized at the cast site, so it arrives with a NON-empty list and
/// is unaffected by this batch's rejection. Pinned rather than argued.
fn t4_an_aura_still_casts_with_its_enchant_target() {
    let registry = CardRegistry::new(all_cards());
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        // `ObjectSpec::card` creates a NAKED object (the documented gotcha, and the very
        // shape that hid this defect behind 42 green fixtures — see the batch's own
        // 44-of-46 census). The Aura synthesis reads the object's LAYER-RESOLVED
        // characteristics, so without enrichment Rancor is not an Aura, carries no
        // `KeywordAbility::Enchant`, and would reach the empty-requirements arm — i.e.
        // this control would fail for a FIXTURE reason and be mistaken for an engine one.
        // The first draft of this test did exactly that.
        .object(enrich_spec_from_def(
            ObjectSpec::card(p(1), "Rancor")
                .with_card_id(CardId("rancor".to_string()))
                .in_zone(ZoneId::Hand(p(1))),
            &defs_by_name(),
        ))
        .object(ObjectSpec::creature(p(1), "Grizzly Bears", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(1));
    if let Ok(pl) = mtg_engine::state::test_util::player_mut(&mut state, p(1)) {
        pl.mana_pool.add(ManaColor::Green, 1);
    }
    let rancor = find(&state, "Rancor");
    let bear = find(&state, "Grizzly Bears");
    let (state, _) = cast(state, p(1), rancor, vec![Target::Object(bear)])
        .expect("CR 303.4a: an Aura still announces its enchant target");
    let so = state.stack_objects().back().expect("Rancor on the stack");
    assert_eq!(
        so.targets.len(),
        1,
        "the Aura's synthesized requirement is non-empty, so it never reaches the arm \
         this batch closed"
    );
    assert!(
        !so.target_requirements.is_empty(),
        "PB-DX20's synthesis is what makes the deleted justification stale — if this is \
         empty, an Aura is reaching the empty-requirements arm again"
    );
}

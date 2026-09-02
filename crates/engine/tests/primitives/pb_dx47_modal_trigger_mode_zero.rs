//! PB-DX47 (`OOS-DX47-3`): a modal `WhenDealsCombatDamageToPlayer` trigger resolves
//! **mode 0**, and it did so on BOTH dispatch paths.
//!
//! # Why this file exists: it corrects an overclaim this batch made about itself
//!
//! PB-DX47 deletes the card-registry dispatch and leaves the runtime lowering
//! (`build_face_ability_vectors`) as the sole dispatcher. That lowering does not
//! carry `ModeSelection` — it pre-selects `modes.first()` as a CR 700.2b bot
//! fallback — so the batch's first draft filed and commented this as *"a real
//! capability the fix gives up"*.
//!
//! **That was an assumption, not a measurement, and it is wrong.** Nothing modal
//! is lost, because nothing modal was ever offered. Three sites, all
//! kind-agnostic:
//!
//! 1. the lowering sets `TriggeredAbilityDef.effect = modes.first()` — mode 0;
//! 2. `flush_sorted` hard-codes `stack_obj.modes_chosen = vec![0]` in **both**
//!    arms of its modal branch, for any `StackObjectKind::TriggeredAbility`,
//!    `Normal` and `CardDefETB` alike — it never consults a player;
//! 3. `resolution.rs`'s modal replacement substitutes `modes.modes[chosen]`, and
//!    it sits OUTSIDE the `is_carddef_etb` branch, so it applies to both kinds —
//!    with `chosen` always `[0]`.
//!
//! And `modal_trigger` (CR 603.3c) is a standing, machine-checked **`AutoChosen`**
//! row in `core::decision_site_walk` — the engine has never offered this choice on
//! any path. The fuzzer prints the same verdict every run: *"a modal TRIGGERED
//! ability's mode is fixed to mode 0 inline"*.
//!
//! So what PB-DX47's deletion changes for the corpus's one modal member
//! (`glissa_sunslayer`, `Completeness::partial`, zero deck-legal exposure) is
//! **one mode-0 resolution instead of two** — which is the double-push, not a
//! regression. `OOS-DX47-3` stays open as the STRUCTURAL gap (`TriggeredAbilityDef`
//! has no `modes` field, so the day CR 603.3c is served the lowering must carry
//! it), with the behavioural delta measured at zero here rather than assumed
//! either way.
//!
//! The subject is a synthetic def rather than `glissa_sunslayer` itself, so this
//! probe keeps measuring the property if that def is ever repaired or re-marked —
//! `core::pb_dx47_dispatch_path_roster::r5b` is what watches the real corpus.

use std::collections::HashMap;

use mtg_engine::cards::card_definition::{
    AbilityDefinition, CardDefinition, Effect, EffectAmount, ModeSelection, PlayerTarget,
    TriggerCondition,
};
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AttackTarget, CardId, CardRegistry, CardType,
    Command, GameStateBuilder, ObjectSpec, PlayerId, Step, TypeLine,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Three modes whose effects are trivially distinguishable by LIFE TOTAL alone,
/// so the assertion never depends on a second subsystem:
/// mode 0 gains 1, mode 1 gains 10, mode 2 gains 100.
fn modal_subject() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dx47-modal-combat-damage".to_string()),
        name: "DX47 Modal Striker".to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Whenever this creature deals combat damage to a player, choose one — \
                      you gain 1 life; or you gain 10 life; or you gain 100 life."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
            effect: Effect::Nothing,
            intervening_if: None,
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                modes: vec![
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(10),
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(100),
                    },
                ],
                mode_costs: None,
                mode_targets: None,
            }),
            trigger_zone: None,
        }],
        ..Default::default()
    }
}

/// **The measurement.** A modal `WhenDealsCombatDamageToPlayer` trigger connects
/// once and gains exactly **1** life — mode 0, once.
///
/// Two numbers matter and they are asserted separately:
/// * `+1` (not `+10`, not `+100`) — the MODE is 0, so nothing modal was lost by
///   the lowering pre-selecting it;
/// * `+1` (not `+2`) — the trigger resolved ONCE, which is PB-DX47's own subject.
///
/// Discriminating revert: restore the deleted card-registry scan in
/// `abilities.rs`'s `CombatDamageDealt` arm — life goes `+1` → `+2`, i.e. mode 0
/// twice, which is exactly what the pre-fix engine did to `glissa_sunslayer` and
/// exactly why "the fix gives up modality" was the wrong reading.
#[test]
fn t1_modal_combat_damage_trigger_resolves_mode_zero_exactly_once() {
    let def = modal_subject();
    let mut defs: HashMap<String, CardDefinition> = all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect();
    defs.insert(def.name.clone(), def.clone());

    let spec = enrich_spec_from_def(
        ObjectSpec::creature(p(1), &def.name, 2, 2).with_card_id(def.card_id.clone()),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(p(1))
        .at_step(Step::DeclareAttackers)
        .object(spec)
        .build()
        .expect("PB-DX47 modal fixture must build");

    // Non-vacuity, stated first: the lowering must actually have produced the
    // runtime trigger, or this probe measures an inert creature swinging.
    let striker = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == def.name)
        .expect("the subject must be on the battlefield");
    assert_eq!(
        striker
            .characteristics
            .triggered_abilities
            .iter()
            .filter(|t| t.trigger_on == mtg_engine::TriggerEvent::SelfDealsCombatDamageToPlayer)
            .count(),
        1,
        "non-vacuity: `enrich_spec_from_def` must have lowered the modal trigger"
    );
    let striker_id = striker.id;
    let life_before = state.players()[&p(1)].life_total;

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p(1),
            attackers: vec![(striker_id, AttackTarget::Player(p(2)))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("DeclareAttackers");

    // Pass whoever actually holds priority, rather than a fixed seat order: the
    // engine hands priority to the ACTOR after a trigger goes on the stack
    // (CR 117.3c, PB-DP1), so a hard-coded `[p1, p2]` loop desyncs the moment the
    // subject's own trigger is placed.
    let mut state = state;
    for _ in 0..24 {
        let Some(holder) = state.turn().priority_holder else {
            break;
        };
        let (s, _) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {holder:?} failed: {e:?}"));
        state = s;
    }

    let gained = state.players()[&p(1)].life_total - life_before;
    assert_eq!(
        gained, 1,
        "PB-DX47 / `OOS-DX47-3`: the modal trigger must resolve MODE 0 (gain 1) \
         exactly ONCE. `+10` or `+100` would mean a different mode was chosen; \
         `+2` would mean the double-push is back."
    );
}

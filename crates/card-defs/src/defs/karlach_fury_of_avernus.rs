// Karlach, Fury of Avernus -- {4}{R}, Legendary Creature -- Tiefling Barbarian 5/4
// Whenever you attack, if it's the first combat phase of the turn, untap all attacking
// creatures, grant first strike until end of turn, add an additional combat phase.
// Choose a Background.
//
// PB-DX1 (2026-08-01): "Whenever you attack" is CR 508.1's controller-scoped attack
// trigger, NOT "whenever Karlach attacks". MCP ruling (2022-06-10, #11) is explicit:
// "Karlach doesn't have to be among the attacking creatures." Fixed from `WhenAttacks`
// (which required her to personally attack -- the exact defect the prior known_wrong
// note named) to `WheneverYouAttack { filter: None }`, which PB-DX1 rows 2/29 both now
// carry `intervening_if` through the runtime lowering, making this combination
// expressible for the first time. `WheneverYouAttack` fires ONCE per combat for the
// controller (abilities.rs's `GameEvent::AttackersDeclared` handler, "Fires per player
// (not per creature), so runs once outside the per-attacker loop"), matching CR 508.1
// exactly -- not once per attacking creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("karlach-fury-of-avernus"),
        name: "Karlach, Fury of Avernus".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            red: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Tiefling", "Barbarian"],
        ),
        oracle_text: "Whenever you attack, if it's the first combat phase of the turn, untap all \
                      attacking creatures. They gain first strike until end of turn. After this \
                      phase, there is an additional combat phase.\nChoose a Background (You can \
                      have a Background as a second commander.)"
            .to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::ChooseABackground),
            // Whenever Karlach attacks (first combat phase only):
            // 1. Untap all attacking creatures.
            // 2. Each attacking creature gains first strike until end of turn.
            // 3. After this phase, there is an additional combat phase.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouAttack { filter: None },
                intervening_if: Some(Condition::IsFirstCombatPhase),
                effect: Effect::Sequence(vec![
                    // Untap all attacking creatures.
                    Effect::ForEach {
                        over: ForEachTarget::EachAttackingCreature,
                        effect: Box::new(Effect::UntapPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                        }),
                    },
                    // Grant first strike to all attacking creatures until end of turn.
                    Effect::ForEach {
                        over: ForEachTarget::EachAttackingCreature,
                        effect: Box::new(Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: EffectLayer::Ability,
                                modification: LayerModification::AddKeyword(
                                    KeywordAbility::FirstStrike,
                                ),
                                filter: EffectFilter::DeclaredTarget { index: 0 },
                                duration: EffectDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        }),
                    },
                    // After this phase, there is an additional combat phase.
                    Effect::AdditionalCombatPhase {
                        followed_by_main: false,
                    },
                ]),
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        // PB-DX1: flipped from known_wrong to Complete (default). All four points of
        // the plan's §6.4 verification hold: (1) every oracle clause -- untap all
        // attacking creatures, grant first strike until end of turn, additional
        // combat phase with NO extra main phase (MCP ruling #1: "doesn't give any
        // additional main phases"), Choose a Background -- is implemented; (2)
        // `WheneverYouAttack { filter: None }` fires once per combat for the
        // controller, not once per attacker (verified in abilities.rs); (3) fail-
        // before probe `test_karlach_fires_without_personally_attacking` /
        // `test_karlach_extra_combat_once_per_turn` in
        // `crates/engine/tests/primitives/pb_dx1_lowered_intervening_if.rs`; (4) no
        // clause failed.
        ..Default::default()
    }
}

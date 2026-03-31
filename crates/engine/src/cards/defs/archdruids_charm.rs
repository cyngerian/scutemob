// Archdruid's Charm — {G}{G}{G}, Instant
// Choose one —
// • Search your library for a creature or land card and reveal it. Put it onto the
//   battlefield tapped if it's a land card. Otherwise, put it into your hand. Then shuffle.
// • Put a +1/+1 counter on target creature you control. It deals damage equal to its power
//   to target creature you don't control.
// • Exile target artifact or enchantment.
//
// Mode 0 TODO: "creature or land" search with conditional destination (battlefield tapped if
// land, hand if creature) — SearchLibrary has a single destination; conditional routing based
// on searched card's type is not in the DSL. DSL gap.
// Mode 1: AddCounters + Bite (expressible).
// Mode 2: ExileObject targeting artifact or enchantment (expressible).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("archdruids-charm"),
        name: "Archdruid's Charm".to_string(),
        mana_cost: Some(ManaCost { green: 3, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Search your library for a creature or land card and reveal it. Put it onto the battlefield tapped if it's a land card. Otherwise, put it into your hand. Then shuffle.\n• Put a +1/+1 counter on target creature you control. It deals damage equal to its power to target creature you don't control.\n• Exile target artifact or enchantment.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            targets: vec![
                // Mode 1 target 0: creature you control (AddCounters + Bite source)
                TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                }),
                // Mode 1 target 1: creature you don't control (Bite target)
                TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::Opponent,
                    ..Default::default()
                }),
                // Mode 2 target 0: artifact or enchantment
                TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                    ..Default::default()
                }),
            ],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Search for creature or land, conditional destination.
                    // TODO: DSL gap — SearchLibrary does not support conditional routing
                    // (land → battlefield tapped, creature → hand). Cannot be faithfully
                    // implemented without a ConditionalDestination variant.
                    Effect::Sequence(vec![]),
                    // Mode 1: +1/+1 counter on creature you control, then it deals damage
                    // equal to its power to a creature you don't control (Bite).
                    Effect::Sequence(vec![
                        Effect::AddCounter {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            counter: CounterType::PlusOnePlusOne,
                            count: 1,
                        },
                        Effect::Bite {
                            source: EffectTarget::DeclaredTarget { index: 0 },
                            target: EffectTarget::DeclaredTarget { index: 1 },
                        },
                    ]),
                    // Mode 2: Exile target artifact or enchantment.
                    Effect::ExileObject {
                        target: EffectTarget::DeclaredTarget { index: 2 },
                    },
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

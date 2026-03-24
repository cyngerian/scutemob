// Howlsquad Heavy — {2}{R}, Creature — Goblin Mercenary 2/3
// Start your engines!
// Other Goblins you control have haste.
// At the beginning of combat on your turn, create a 1/1 red Goblin creature token.
//   That token attacks this combat if able.
// Max speed — {T}: Add {R} for each Goblin you control.
//
// TODO: Multiple DSL gaps:
//   (1) "Start your engines!" — vehicle/speed mechanic not in DSL
//   (2) "At the beginning of combat, create a token that attacks if able" — combat trigger
//       with forced-attack token not expressible
//   (3) "Max speed" — speed mechanic not in DSL
// Implementing: Haste grant to Goblins, mana tap ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("howlsquad-heavy"),
        name: "Howlsquad Heavy".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Mercenary"]),
        oracle_text: "Start your engines!\nOther Goblins you control have haste.\nAt the beginning of combat on your turn, create a 1/1 red Goblin creature token. That token attacks this combat if able.\nMax speed — {T}: Add {R} for each Goblin you control.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            // "Other Goblins you control have haste." — Layer 6 keyword grant
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Goblin".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: "At the beginning of combat, create 1/1 Goblin that must attack" (combat trigger + forced attack)
            // "Max speed — {T}: Add {R} for each Goblin you control."
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaScaled {
                    player: PlayerTarget::Controller,
                    color: ManaColor::Red,
                    count: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_subtype: Some(SubType("Goblin".to_string())),
                            controller: TargetController::You,
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}

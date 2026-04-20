// Druid Class — {1}{G}, Enchantment — Class
// CR 716: Class leveling mechanic with 3 levels.
// Level 1 (always active): Landfall — Whenever a land you control enters, you gain 1 life.
// Level 2 ({2}{G}): You may play an additional land on each of your turns.
// Level 3 ({4}{G}): When this Class becomes level 3, target land becomes a creature.
// Level 1 Landfall trigger: CR 207.2c — ability word, no dedicated CR rule.
// Implemented via TriggerCondition::WheneverPermanentEntersBattlefield { Land + You }.
// See jaddi_offshoot.rs for the canonical template.
// TODO: Level 3 needs land-animation continuous effect + ETB trigger on level-up
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("druid-class"),
        name: "Druid Class".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Enchantment], &["Class"]),
        oracle_text: "(Gain the next level as a sorcery to add its ability.)\nLandfall — Whenever a land you control enters, you gain 1 life.\n{2}{G}: Level 2\nYou may play an additional land on each of your turns.\n{4}{G}: Level 3\nWhen this Class becomes level 3, target land you control becomes a creature with haste and \"This creature's power and toughness are each equal to the number of lands you control.\" It's still a land.".to_string(),
        abilities: vec![
            // Level 1 ability (always active, not a ClassLevel bar):
            // Landfall — Whenever a land you control enters, you gain 1 life.
            // CR 207.2c: Landfall is an ability word; uses WheneverPermanentEntersBattlefield.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // Level 2 bar: {2}{G}: You may play an additional land on each of your turns.
            // CR 305.2: When level 2 is reached, register AdditionalLandPlays static.
            AbilityDefinition::ClassLevel {
                level: 2,
                cost: ManaCost {
                    generic: 2,
                    green: 1,
                    ..Default::default()
                },
                abilities: vec![AbilityDefinition::AdditionalLandPlays { count: 1 }],
            },
            // Level 3 bar: {4}{G}: Animate a land.
            // TODO: Needs land-animation continuous effect
            AbilityDefinition::ClassLevel {
                level: 3,
                cost: ManaCost {
                    generic: 4,
                    green: 1,
                    ..Default::default()
                },
                abilities: vec![],
            },
        ],
        ..Default::default()
    }
}

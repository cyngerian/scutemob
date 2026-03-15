// Druid Class — {1}{G}, Enchantment — Class
// CR 716: Class leveling mechanic with 3 levels.
// Level 1 (always active): Landfall — Whenever a land you control enters, you gain 1 life.
// Level 2 ({2}{G}): You may play an additional land on each of your turns.
// Level 3 ({4}{G}): When this Class becomes level 3, target land becomes a creature.
// TODO: Level 1 Landfall trigger needs LandEntersBattlefield trigger condition
// TODO: Level 2 needs "additional land play" modifier (not in DSL)
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
            // TODO: Needs LandEntersBattlefield trigger condition for proper Landfall
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
            // Level 2 bar: {2}{G}: You may play an additional land.
            // TODO: Needs "additional land play" effect (not in DSL)
            AbilityDefinition::ClassLevel {
                level: 2,
                cost: ManaCost {
                    generic: 2,
                    green: 1,
                    ..Default::default()
                },
                abilities: vec![],
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

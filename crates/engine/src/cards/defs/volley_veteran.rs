// Volley Veteran — {3}{R}, Creature — Goblin Warrior 4/2
// When this creature enters, it deals damage to target creature an opponent controls
//   equal to the number of Goblins you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("volley-veteran"),
        name: "Volley Veteran".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "When this creature enters, it deals damage to target creature an opponent controls equal to the number of Goblins you control.".to_string(),
        power: Some(4),
        toughness: Some(2),
        abilities: vec![
            // CR 603.1: ETB trigger — deal damage equal to number of Goblins you control.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            has_subtype: Some(SubType("Goblin".to_string())),
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}

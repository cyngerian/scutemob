// Voice of Victory — {1}{W}, Creature — Human Bard 1/3
// Mobilize 2 (Whenever this creature attacks, create two tapped and attacking 1/1 red
// Warrior creature tokens. Sacrifice them at the beginning of the next end step.)
// Your opponents can't cast spells during your turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("voice-of-victory"),
        name: "Voice of Victory".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Bard"]),
        oracle_text: "Mobilize 2 (Whenever this creature attacks, create two tapped and attacking 1/1 red Warrior creature tokens. Sacrifice them at the beginning of the next end step.)\nYour opponents can't cast spells during your turn.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            // Mobilize 2: create 2 tokens on attack
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Warrior".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Warrior".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 2,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: true,
                        enters_attacking: true,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        sacrifice_at_end_step: true, // Mobilize: sacrifice at next end step
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "Opponents can't cast during your turn" stax restriction not in DSL.
        ],
        ..Default::default()
    }
}

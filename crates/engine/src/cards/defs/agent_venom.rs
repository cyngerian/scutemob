// Agent Venom — {2}{B}, Legendary Creature — Symbiote Soldier Hero 2/3
// Flash
// Menace
// Whenever another nontoken creature you control dies, you draw a card and lose 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("agent-venom"),
        name: "Agent Venom".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Symbiote", "Soldier", "Hero"],
        ),
        oracle_text: "Flash\nMenace\nWhenever another nontoken creature you control dies, you draw a card and lose 1 life.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // TODO: "Another nontoken creature you control dies" — WheneverCreatureDies overbroad.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies,
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::LoseLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}

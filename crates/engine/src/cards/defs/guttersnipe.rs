// Guttersnipe — {2}{R}, Creature — Goblin Shaman 2/2
// Whenever you cast an instant or sorcery spell, this creature deals 2 damage to each opponent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("guttersnipe"),
        name: "Guttersnipe".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Shaman"]),
        oracle_text: "Whenever you cast an instant or sorcery spell, this creature deals 2 damage to each opponent.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // Instant/sorcery spell filter applied.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Instant, CardType::Sorcery]),
                    noncreature_only: false,
                },
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(2),
                    }),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}

// Archmage of Runes — {3}{U}{U}, Creature — Giant Wizard 3/6
// Instant and sorcery spells you cast cost {1} less to cast.
// Whenever you cast an instant or sorcery spell, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("archmage-of-runes"),
        name: "Archmage of Runes".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 2, ..Default::default() }),
        types: creature_types(&["Giant", "Wizard"]),
        oracle_text: "Instant and sorcery spells you cast cost {1} less to cast.\nWhenever you cast an instant or sorcery spell, draw a card.".to_string(),
        power: Some(3),
        toughness: Some(6),
        // CR 601.2f: Instant and sorcery spells controller casts cost {1} less.
        // Uses SpellCostFilter::InstantOrSorcery (already existed pre-PB-29).
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::InstantOrSorcery,
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
        }],
        abilities: vec![
            // Whenever you cast an instant or sorcery spell, draw a card.
            // Instant/sorcery spell filter applied.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Instant, CardType::Sorcery]),
                    noncreature_only: false,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}

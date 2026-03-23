// Baral, Chief of Compliance — {1}{U}, Legendary Creature — Human Wizard 1/3
// Instant and sorcery spells you cast cost {1} less to cast.
// Whenever a spell or ability you control counters a spell, you may draw a card.
// If you do, discard a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("baral-chief-of-compliance"),
        name: "Baral, Chief of Compliance".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Wizard"],
        ),
        oracle_text: "Instant and sorcery spells you cast cost {1} less to cast.\nWhenever a spell or ability you control counters a spell, you may draw a card. If you do, discard a card.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            // TODO: "Whenever you counter a spell" trigger not in DSL.
        ],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::InstantOrSorcery,
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
        }],
        ..Default::default()
    }
}

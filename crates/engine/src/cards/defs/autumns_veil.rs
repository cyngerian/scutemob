// Autumn's Veil — {G}, Instant
// Spells you control can't be countered by blue or black spells this turn, and
// creatures you control can't be the targets of blue or black spells this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("autumns-veil"),
        name: "Autumn's Veil".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Spells you control can't be countered by blue or black spells this turn, and creatures you control can't be the targets of blue or black spells this turn.".to_string(),
        abilities: vec![
            // TODO: Color-scoped counter protection + hexproof from color sources.
            // Neither "can't be countered by [color] spells" nor "can't be targeted
            // by [color] spells" effects exist in the DSL.
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

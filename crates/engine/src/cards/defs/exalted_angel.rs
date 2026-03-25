// Exalted Angel — {3}{W}{W}{W}, Creature — Angel 4/5
// Flying, lifelink.
// Morph {2}{W}{W} (You may cast this card face down as a 2/2 creature for {3}.
// Turn it face up any time for its morph cost.)
//
// AbilityDefinition::Morph carries the turn-face-up cost {2}{W}{W}.
// KeywordAbility::Morph is the marker for quick presence-checking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("exalted-angel"),
        name: "Exalted Angel".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 3, ..Default::default() }),
        types: types_sub(&[CardType::Creature], &["Angel"]),
        oracle_text:
            "Flying, lifelink.\n\
             Morph {2}{W}{W} (You may cast this card face down as a 2/2 creature for {3}. \
             Turn it face up any time for its morph cost.)"
                .to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            AbilityDefinition::Keyword(KeywordAbility::Morph),
            AbilityDefinition::Morph { cost: ManaCost { generic: 2, white: 2, ..Default::default() } },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        activated_ability_cost_reductions: vec![],
    }
}

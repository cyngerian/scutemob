// Akroma, Angel of Fury — {5}{R}{R}{R}, Legendary Creature — Angel 6/6
// Flying, trample, protection from white and from blue.
// Akroma, Angel of Fury can't be countered.
// {R}: Akroma, Angel of Fury gets +1/+0 until end of turn.
// Morph {R}{R}{R} (You may cast this card face down as a 2/2 creature for {3}.
// Turn it face up any time for its morph cost.)
//
// "Can't be countered" and the {R} pump activated ability are DSL gaps — omitted.
// Protection from white and blue expressed as two Protection entries.
// AbilityDefinition::Morph carries the turn-face-up cost {R}{R}{R}.
// KeywordAbility::Morph is the marker for quick presence-checking.
use crate::cards::helpers::*;
use crate::state::types::ProtectionQuality;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("akroma-angel-of-fury"),
        name: "Akroma, Angel of Fury".to_string(),
        mana_cost: Some(ManaCost { generic: 5, red: 3, ..Default::default() }),
        types: types_sub(&[CardType::Creature], &["Angel"]),
        oracle_text:
            "Flying, trample, protection from white and from blue.\n\
             Akroma, Angel of Fury can't be countered.\n\
             {R}: Akroma, Angel of Fury gets +1/+0 until end of turn.\n\
             Morph {R}{R}{R}"
                .to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromColor(Color::White),
            )),
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromColor(Color::Blue),
            )),
            // "Can't be countered" — DSL gap, omitted.
            // {R}: +1/+0 activated ability — DSL gap (no self-pump with color cost), omitted.
            AbilityDefinition::Keyword(KeywordAbility::Morph),
            AbilityDefinition::Morph { cost: ManaCost { red: 3, ..Default::default() } },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}

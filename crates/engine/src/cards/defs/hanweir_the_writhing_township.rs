// Hanweir, the Writhing Township — Melded permanent definition
// CR 712.5b: Hanweir Garrison and Hanweir Battlements meld to form this.
//
// This CardDefinition exists solely to hold the melded back_face characteristics.
// It is never cast directly. Both Hanweir Garrison and Hanweir Battlements reference
// this card_id in their meld_pair.melded_card_id.
//
// Melded face: Legendary Creature — Eldrazi Ooze, 7/4, Trample, Haste
// "Whenever Hanweir, the Writhing Township attacks, create two 3/2 colorless
//  Eldrazi Horror creature tokens that are tapped and attacking."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hanweir-the-writhing-township"),
        name: "Hanweir, the Writhing Township".to_string(),
        mana_cost: None,
        types: TypeLine::default(),
        oracle_text: String::new(),
        abilities: vec![],
        power: None,
        toughness: None,
        back_face: Some(CardFace {
            name: "Hanweir, the Writhing Township".to_string(),
            mana_cost: None,
            types: full_types(
                &[SuperType::Legendary],
                &[CardType::Creature],
                &["Eldrazi", "Ooze"],
            ),
            oracle_text: "Trample, haste\nWhenever Hanweir, the Writhing Township attacks, create two 3/2 colorless Eldrazi Horror creature tokens that are tapped and attacking.".to_string(),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Trample),
                AbilityDefinition::Keyword(KeywordAbility::Haste),
                // TODO: attack trigger — create two 3/2 colorless Eldrazi Horror creature tokens
                // tapped and attacking. Requires "tapped and attacking" token creation.
            ],
            power: Some(7),
            toughness: Some(4),
            color_indicator: None,
        }),
        ..Default::default()
    }
}

// 55 (additional). Adrix and Nev, Twincasters — {2}{G}{U}, Legendary Creature — Merfolk
// Wizard 2/2. Ward {2}. If one or more tokens would be created under your control,
// twice that many of those tokens are created instead.
//
// Ward {2} is encoded as KeywordAbility::Ward(2); the triggered ability that counters
// spells/abilities targeting this creature unless the opponent pays {2} is generated
// automatically from the keyword by state/builder.rs.
//
// The token-doubling ability is a replacement effect (CR 614.1): "If one or more tokens
// would be created under your control, twice that many of those tokens are created
// instead." ReplacementTrigger::WouldCreateToken does not yet exist in the DSL.
// TODO: Add ReplacementTrigger::WouldCreateToken { player_filter: PlayerFilter }
// and ReplacementModification::DoubleTokens to replacement_effect.rs, then replace
// this TODO with AbilityDefinition::Replacement using those variants.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("adrix-and-nev-twincasters"),
        name: "Adrix and Nev, Twincasters".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, blue: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Merfolk", "Wizard"]),
        oracle_text:
            "Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)\n\
             If one or more tokens would be created under your control, twice that many of those tokens are created instead."
                .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 702.21a: Ward {2} — generates a triggered ability at object-construction
            // time that counters any spell or ability an opponent controls that targets this
            // creature, unless that opponent pays {2}.
            AbilityDefinition::Keyword(KeywordAbility::Ward(2)),
            // TODO: Token-doubling replacement effect — requires ReplacementTrigger::WouldCreateToken
            // and ReplacementModification::DoubleTokens (not yet in DSL). See note above.
        ],
        back_face: None,
    }
}

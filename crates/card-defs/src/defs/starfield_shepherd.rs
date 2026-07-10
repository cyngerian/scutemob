// Starfield Shepherd — {3}{W}{W}, Creature — Angel 3/2
// Flying
// When this creature enters, search your library for a basic Plains card or a creature
// card with mana value 1 or less, reveal it, put it into your hand, then shuffle.
// Warp {1}{W}
//
// Warp primitive shipped in PB-AC5 (KeywordAbility::Warp + AltCostKind::Warp). Card
// still BLOCKED: the ETB search is "basic Plains card" OR "creature card with mana
// value 1 or less" — TargetFilter cannot express cross-group OR (a Plains-subtype
// filter OR an independent creature-type + max-mana-value filter). Do not author until
// that disjunctive-filter primitive exists.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("starfield-shepherd"),
        name: "Starfield Shepherd".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 2, ..Default::default() }),
        types: creature_types(&["Angel"]),
        oracle_text: "Flying\nWhen this creature enters, search your library for a basic Plains card or a creature card with mana value 1 or less, reveal it, put it into your hand, then shuffle.\nWarp {1}{W} (You may cast this card from your hand for its warp cost. Exile this creature at the beginning of the next end step, then you may cast it from exile on a later turn.)".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Warp),
            // CR 702.185a: Warp {1}{W}. No non-mana cost components; hand-only (no
            // graveyard-cast permission is granted in the oracle text).
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Warp,
                cost: ManaCost { generic: 1, white: 1, ..Default::default() },
                details: Some(AltCastDetails::Warp {
                    costs: vec![],
                    from_graveyard: false,
                }),
            },
            // ENGINE-BLOCKED: ETB — search for "basic Plains OR creature with MV ≤ 1".
            // TargetFilter cannot express: (1) basic Plains subtype filter, OR (2) creature
            // with max_mana_value ≤ 1. Two-filter OR semantics and mana value ceiling both
            // missing from TargetFilter. DSL gap (separate primitive from PB-AC5's Warp).
        ],
        completeness: Completeness::partial("ETB — search for 'basic Plains OR creature with MV ≤ 1'. TargetFilter cannot express: (1) basic Plains subtype filter..."),
        ..Default::default()
    }
}

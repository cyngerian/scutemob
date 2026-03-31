// Starfield Shepherd — {3}{W}{W}, Creature — Angel 3/2
// Flying
// When this creature enters, search your library for a basic Plains card or a creature
// card with mana value 1 or less, reveal it, put it into your hand, then shuffle.
// Warp {1}{W}
//
// TODO: Warp keyword is not in the DSL (KeywordAbility enum). No AltCostKind::Warp exists.
// TODO: ETB search filter requires OR semantics: "basic Plains" OR "creature MV ≤ 1".
// TargetFilter has no max_mana_value field and no OR-combination of two independent filters.
// Per W5 policy, abilities that can't be faithfully expressed are left as TODO.
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
            // TODO: ETB — search for "basic Plains OR creature with MV ≤ 1".
            // TargetFilter cannot express: (1) basic Plains subtype filter, OR (2) creature
            // with max_mana_value ≤ 1. Two-filter OR semantics and mana value ceiling both
            // missing from TargetFilter. DSL gap.
            // TODO: Warp {1}{W} — AltCostKind::Warp does not exist in the DSL.
        ],
        ..Default::default()
    }
}

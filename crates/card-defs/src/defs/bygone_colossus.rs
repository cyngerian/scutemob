// Bygone Colossus — {9}, Artifact Creature — Robot Giant 9/9
// Warp {3} (You may cast this card from your hand for its warp cost. Exile this creature at
// the beginning of the next end step, then you may cast it from exile on a later turn.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bygone-colossus"),
        name: "Bygone Colossus".to_string(),
        mana_cost: Some(ManaCost {
            generic: 9,
            ..Default::default()
        }),
        types: types_sub(
            &[CardType::Artifact, CardType::Creature],
            &["Robot", "Giant"],
        ),
        oracle_text: "Warp {3} (You may cast this card from your hand for its warp cost. Exile \
                      this creature at the beginning of the next end step, then you may cast it \
                      from exile on a later turn.)"
            .to_string(),
        power: Some(9),
        toughness: Some(9),
        abilities: vec![
            // CR 702.185a: Warp keyword marker for presence-checking.
            AbilityDefinition::Keyword(KeywordAbility::Warp),
            // CR 702.185a: Warp {3} — cast from hand for {3} instead of {9}.
            // No non-mana costs, and no "cast from graveyard" permission (unlike Timeline
            // Culler's warp).
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Warp,
                cost: ManaCost {
                    generic: 3,
                    ..Default::default()
                },
                details: Some(AltCastDetails::Warp {
                    costs: vec![],
                    from_graveyard: false,
                }),
            },
        ],
        ..Default::default()
    }
}

// Perennial Behemoth — {5}, Artifact Creature — Beast 2/7.
// You may play lands from your graveyard.
// Unearth {G}{G}.
//
// CR 601.3, CR 305.1: Graveyard land play implemented via StaticPlayFromGraveyard (PB-B).
// Unearth keyword marker present; AltCastAbility for graveyard activation is handled by
// casting.rs via AltCostKind::Unearth when on the battlefield (standard Unearth pattern).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("perennial-behemoth"),
        name: "Perennial Behemoth".to_string(),
        mana_cost: Some(ManaCost { generic: 5, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Beast"]),
        oracle_text: "You may play lands from your graveyard.\nUnearth {G}{G} ({G}{G}: Return this card from your graveyard to the battlefield. It gains haste. Exile it at the beginning of the next end step or if it would leave the battlefield. Unearth only as a sorcery.)".to_string(),
        power: Some(2),
        toughness: Some(7),
        abilities: vec![
            // CR 601.3, CR 305.1: "You may play lands from your graveyard."
            // Registers a PlayFromGraveyardPermission (LandsOnly filter) when this permanent
            // enters the battlefield. Cleaned up when Perennial Behemoth leaves.
            AbilityDefinition::StaticPlayFromGraveyard {
                filter: PlayFromTopFilter::LandsOnly,
                condition: None,
            },
            // CR 702.83a: Unearth keyword marker for presence-checking.
            AbilityDefinition::Keyword(KeywordAbility::Unearth),
            // CR 702.83a: Unearth activated ability — {G}{G}: Return from graveyard.
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Unearth,
                cost: ManaCost { green: 2, ..Default::default() },
                details: None,
            },
        ],
        ..Default::default()
    }
}

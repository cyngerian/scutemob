// Perennial Behemoth — {5}, Artifact Creature — Beast 2/7.
// You may play lands from your graveyard.
// Unearth {G}{G}.
// TODO: DSL gap — "play lands from graveyard" static not expressible; Unearth is a
// keyword with activated ability from graveyard (AltCostKind::Unearth) but the engine
// would need the graveyard zone as activation location.
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
            AbilityDefinition::Keyword(KeywordAbility::Unearth),
        ],
        // TODO: "play lands from graveyard" static; Unearth activation from graveyard
        ..Default::default()
    }
}

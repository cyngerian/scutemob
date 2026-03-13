// Ancient Greenwarden — {4}{G}{G}, Creature — Elemental 5/7
// Reach.
// You may play lands from your graveyard.
// If a land entering causes a triggered ability of a permanent you control to trigger,
// that ability triggers an additional time.
//
// Reach is implemented.
//
// TODO: DSL gap — "you may play lands from your graveyard" requires a zone-play
// permission grant (graveyard → land play) with no current DSL primitive.
// TODO: DSL gap — the land-ETB trigger doubling requires a continuous effect that
// intercepts trigger generation and duplicates it; no DSL primitive exists.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ancient-greenwarden"),
        name: "Ancient Greenwarden".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            green: 2,
            ..Default::default()
        }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elemental"]),
        oracle_text: "Reach\nYou may play lands from your graveyard.\nIf a land entering causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.".to_string(),
        power: Some(5),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Reach),
        ],
        ..Default::default()
    }
}

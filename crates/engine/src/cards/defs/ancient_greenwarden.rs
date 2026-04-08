// Ancient Greenwarden — {4}{G}{G}, Creature — Elemental 5/7
// Reach.
// You may play lands from your graveyard.
// If a land entering causes a triggered ability of a permanent you control to trigger,
// that ability triggers an additional time.
//
// CR 601.3, CR 305.1: Graveyard land play implemented via StaticPlayFromGraveyard (PB-B).
// TODO: The land-ETB trigger doubling ("that ability triggers an additional time") requires a
//       continuous effect that intercepts trigger generation and duplicates land-ETB triggers.
//       This is a separate primitive (similar to Panharmonicon but filtered to land ETBs).
//       Deferred — no DSL primitive exists for conditional trigger doubling. (See PB-M plan.)
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
            // CR 601.3, CR 305.1: "You may play lands from your graveyard."
            // Registers a PlayFromGraveyardPermission (LandsOnly filter) when this permanent
            // enters the battlefield. Cleaned up when Ancient Greenwarden leaves.
            AbilityDefinition::StaticPlayFromGraveyard {
                filter: PlayFromTopFilter::LandsOnly,
                condition: None,
            },
        ],
        ..Default::default()
    }
}

// Forsaken Miner — {B}, Creature — Skeleton Rogue 2/2
// This creature can't block.
// Whenever you commit a crime, you may pay {B}. If you do, return this card from your
// graveyard to the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("forsaken-miner"),
        name: "Forsaken Miner".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Skeleton", "Rogue"]),
        oracle_text: "This creature can't block.\nWhenever you commit a crime, you may pay {B}. If you do, return this card from your graveyard to the battlefield. (Targeting opponents, anything they control, and/or cards in their graveyards is a crime.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CantBlock),
            // TODO: "Whenever you commit a crime" — TriggerCondition::WheneverYouCommitACrime
            // does not exist in the DSL. The crime mechanic (CR 701.59) requires tracking when
            // you target an opponent, something they control, or a card in their graveyard.
            // Additionally, the "may pay {B}" optional cost at trigger resolution is not
            // expressible. Empty per W5 policy.
        ],
        ..Default::default()
    }
}

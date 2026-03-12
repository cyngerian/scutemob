// Cult Conscript
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cult-conscript"),
        name: "Cult Conscript".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Skeleton", "Warrior"]),
        oracle_text: "This creature enters tapped.\n{1}{B}: Return this card from your graveyard to the battlefield. Activate only if a non-Skeleton creature died under your control this turn.".to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — this creature enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
            },
            // TODO: Activated — {1}{B}: Return this card from your graveyard to the battlefield.
            // Activate only if a non-Skeleton creature died under your control this turn.
            // DSL gap: graveyard-zone activated ability with death_trigger_filter condition.
        ],
        power: Some(2),
        toughness: Some(1),
        ..Default::default()
    }
}

// Spymaster's Vault
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spymasters-vault"),
        name: "Spymaster's Vault".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control a Swamp.\n{T}: Add {B}.\n{B}, {T}: Target creature you control connives X, where X is the number of creatures that died this turn. (Draw X cards, then discard X cards. Put a +1/+1 counter on that creature for each nonland card discarded this way.)".to_string(),
        abilities: vec![
            // TODO: This land enters tapped unless you control a Swamp.
            // DSL gap: conditional ETB tapped (no condition field on ReplacementModification::EntersTapped).
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
            },
            // TODO: Activated — {B}, {T}: Target creature connives X (where X = creatures died this turn).
            // DSL gap: targeted_trigger + death_count tracking per turn.
        ],
        ..Default::default()
    }
}

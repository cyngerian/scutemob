// The Soul Stone — {1}{B} Legendary Artifact — Infinity Stone; Indestructible; {T}: Add {B};
// {6}{B},{T},Exile a creature you control: Harness; ∞ — upkeep return creature from graveyard.
// TODO: "Harness The Soul Stone" is a custom Infinity Stone mechanic (digital-only?).
// The ∞ ability (at beginning of upkeep, return target creature from graveyard) requires
// the Harness activation to be tracked as a permanent state flag — DSL gap.
// Implementing just the Indestructible keyword and {T}: Add {B}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-soul-stone"),
        name: "The Soul Stone".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Artifact], &["Infinity Stone"]),
        oracle_text: "Indestructible\n{T}: Add {B}.\n{6}{B}, {T}, Exile a creature you control: Harness The Soul Stone. (Once harnessed, its ∞ ability is active.)\n∞ — At the beginning of your upkeep, return target creature card from your graveyard to the battlefield.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

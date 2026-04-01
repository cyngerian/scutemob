// By Invitation Only — {3}{W}{W}, Sorcery
// Choose a number between 0 and 13. Each player sacrifices that many creatures of
// their choice.
//
// TODO: "Choose a number" — interactive number choice deferred to M10.
// Approximated as SacrificePermanents(EachPlayer, 1) as a reasonable default.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("by-invitation-only"),
        name: "By Invitation Only".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose a number between 0 and 13. Each player sacrifices that many creatures of their choice.".to_string(),
        abilities: vec![
            // TODO: Interactive number choice (0-13) deferred to M10.
            // Each player sacrifices that many creatures.
            AbilityDefinition::Spell {
                effect: Effect::SacrificePermanents {
                    player: PlayerTarget::EachPlayer,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

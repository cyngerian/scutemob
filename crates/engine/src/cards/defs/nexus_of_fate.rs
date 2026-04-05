// Nexus of Fate — {5}{U}{U}, Instant
// Take an extra turn after this one.
// If Nexus of Fate would be put into a graveyard from anywhere, reveal Nexus of
// Fate and shuffle it into its owner's library instead.
//
// CR 500.7: "Take an extra turn after this one." — grants one extra turn to controller.
// CR 614.1a: The graveyard-to-library replacement applies from anywhere (hand, stack,
// graveyard itself, etc.). Implemented via self_shuffle_on_resolution for the
// resolution-destination case. Other zones (discard, mill) are deferred to full
// replacement-effect infrastructure.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nexus-of-fate"),
        name: "Nexus of Fate".to_string(),
        mana_cost: Some(ManaCost { generic: 5, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Take an extra turn after this one.\nIf Nexus of Fate would be put into a graveyard from anywhere, reveal Nexus of Fate and shuffle it into its owner's library instead.".to_string(),
        abilities: vec![
            // CR 500.7: Take an extra turn after this one.
            AbilityDefinition::Spell {
                effect: Effect::ExtraTurn {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
            // TODO: CR 614.1a — "from anywhere" graveyard replacement (discard, mill, counter)
            // needs full replacement infrastructure for non-permanents. Only resolution case
            // is handled via self_shuffle_on_resolution flag.
        ],
        // CR 614.1a: Shuffle into library instead of going to graveyard on resolution.
        self_shuffle_on_resolution: true,
        ..Default::default()
    }
}

// Sylvan Messenger — {3}{G}, Creature — Elf 2/2
// Trample (This creature can deal excess combat damage to the player or planeswalker it's
// attacking.)
// When this creature enters, reveal the top four cards of your library. Put all Elf cards
// revealed this way into your hand and the rest on the bottom of your library in any order.
//
// The non-Elf cards go to the bottom in a fixed order rather than a player-chosen order
// (RevealAndRoute has no per-card ordering choice) — same shape as goblin_ringleader.rs,
// which ships this way as Complete.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sylvan-messenger"),
        name: "Sylvan Messenger".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Elf"]),
        oracle_text: "Trample (This creature can deal excess combat damage to the player or \
                      planeswalker it's attacking.)\nWhen this creature enters, reveal the top \
                      four cards of your library. Put all Elf cards revealed this way into your \
                      hand and the rest on the bottom of your library in any order."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::RevealAndRoute {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(4),
                    filter: TargetFilter {
                        has_subtype: Some(SubType("Elf".to_string())),
                        ..Default::default()
                    },
                    matched_dest: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                    unmatched_dest: ZoneTarget::Library {
                        owner: PlayerTarget::Controller,
                        position: LibraryPosition::Bottom,
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}

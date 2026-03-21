// Goblin Ringleader — {3}{R}, Creature — Goblin 2/2
// Haste
// When this creature enters, reveal the top four cards of your library. Put all
// Goblin cards revealed this way into your hand and the rest on the bottom of
// your library in any order.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-ringleader"),
        name: "Goblin Ringleader".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Haste (This creature can attack and {T} as soon as it comes under your control.)\nWhen this creature enters, reveal the top four cards of your library. Put all Goblin cards revealed this way into your hand and the rest on the bottom of your library in any order.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::RevealAndRoute {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(4),
                    filter: TargetFilter {
                        has_subtype: Some(SubType("Goblin".to_string())),
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
            },
        ],
        ..Default::default()
    }
}

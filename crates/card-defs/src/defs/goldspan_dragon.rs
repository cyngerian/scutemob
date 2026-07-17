// Goldspan Dragon — {3}{R}{R}, Creature — Dragon 4/4
// Flying, haste
// Whenever this creature attacks or becomes the target of a spell, create a Treasure token.
// Treasures you control have "{T}, Sacrifice this artifact: Add two mana of any one color."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goldspan-dragon"),
        name: "Goldspan Dragon".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 2,
            ..Default::default()
        }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying, haste\nWhenever this creature attacks or becomes the target of a \
                      spell, create a Treasure token.\nTreasures you control have \"{T}, \
                      Sacrifice this artifact: Add two mana of any one color.\""
            .to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // Attack trigger: create Treasure
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // "…or becomes the target of a spell" (PB-AC6). Modeled as a second triggered
            // ability rather than one ability with two trigger events: the oracle ability
            // triggers once per qualifying event, so the observable behavior is identical
            // (attacking AND being targeted in one turn yields two Treasures either way).
            // scope: None = the source itself; by_opponent: false = any controller;
            // include_abilities: false = spells only, per the oracle.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenBecomesTarget {
                    scope: None,
                    by_opponent: false,
                    include_abilities: false,
                },
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // ENGINE-BLOCKED: "Treasures you control have '{T}, Sacrifice: Add two mana of any
            // one color.'" — a static ability-granting override that replaces the Treasure's
            // own printed mana ability. No static grant to a filtered set of permanents that
            // can override an existing activated ability's mana output.
        ],
        completeness: Completeness::partial(
            "'Treasures you control have '{T}, Sacrifice: Add two mana of any one color.'' — a \
             static ability-granting override that...",
        ),
        ..Default::default()
    }
}

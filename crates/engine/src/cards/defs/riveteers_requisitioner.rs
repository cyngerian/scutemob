// Riveteers Requisitioner — {1}{R}, Creature — Lizard Rogue 3/1
// Blitz {2}{R}; when it dies, create a Treasure token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("riveteers-requisitioner"),
        name: "Riveteers Requisitioner".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Lizard", "Rogue"]),
        oracle_text: "When this creature dies, create a Treasure token. (It's an artifact with \"{T}, Sacrifice this token: Add one mana of any color.\")\nBlitz {2}{R} (If you cast this spell for its blitz cost, it gains haste and \"When this creature dies, draw a card.\" Sacrifice it at the beginning of the next end step.)".to_string(),
        power: Some(3),
        toughness: Some(1),
        abilities: vec![
            // CR 702.152: Blitz {2}{R} — alternative cost granting haste,
            // sacrifice at end step, and draw-on-death trigger.
            // The Keyword marker is required for quick presence-checking.
            AbilityDefinition::Keyword(KeywordAbility::Blitz),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Blitz,
                cost: ManaCost { generic: 2, red: 1, ..Default::default() },
                details: None,
            },
            // CR 603.1: When Riveteers Requisitioner dies, create a Treasure token.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}

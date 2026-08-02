// Prosperous Innkeeper — {1}{G}, Creature — Halfling Citizen 1/1
// When this creature enters, create a Treasure token.
// Whenever another creature you control enters, you gain 1 life.
//
// CARDS-2 (scutemob-181) second fix cycle: the header and oracle_text previously
// invented an "Alliance —" ability word this printing does not carry (verified via
// sqlite cards.sqlite; the MCP lookup independently confirms no ability word). Removed
// from both. The implementation itself — WheneverCreatureEntersBattlefield with
// controller: You and exclude_self: true (PB-XS-E, CR 109.1 / 603.2) — was already
// correct for the plain "whenever another creature you control enters" trigger.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("prosperous-innkeeper"),
        name: "Prosperous Innkeeper".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Halfling", "Citizen"]),
        oracle_text: "When this creature enters, create a Treasure token. (It's an artifact with \
                      \"{T}, Sacrifice this token: Add one mana of any color.\")\nWhenever \
                      another creature you control enters, you gain 1 life."
            .to_string(),
        abilities: vec![
            // CR 603.1: ETB trigger — create a Treasure token.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // "Whenever another creature you control enters, you gain 1 life."
            // exclude_self: true (PB-XS-E) prevents Prosperous Innkeeper's own ETB from
            // triggering this ability (CR 109.1 / 603.2 — "another").
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: true,
                },
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        power: Some(1),
        toughness: Some(1),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    }
}

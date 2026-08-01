// Contaminant Grafter — {4}{G}, Creature — Phyrexian Druid 5/5
// Trample, toxic 1
// Whenever one or more creatures you control deal combat damage to one or more players,
// proliferate.
// Corrupted — At the beginning of your end step, if an opponent has three or more poison
// counters, draw a card, then you may put a land card from your hand onto the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("contaminant-grafter"),
        name: "Contaminant Grafter".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Phyrexian", "Druid"]),
        oracle_text: "Trample, toxic 1\nWhenever one or more creatures you control deal combat \
                      damage to one or more players, proliferate.\nCorrupted \u{2014} At the \
                      beginning of your end step, if an opponent has three or more poison \
                      counters, draw a card, then you may put a land card from your hand onto the \
                      battlefield."
            .to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::Toxic(1)),
            // CR 510.3a / CR 603.2c: "Whenever one or more creatures you control deal combat
            // damage to one or more players, proliferate." — batch trigger.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition:
                    TriggerCondition::WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer {
                        filter: None,
                    },
                effect: Effect::Proliferate,
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // Corrupted — at the beginning of your end step, if an opponent has 3+ poison counters,
            // draw a card, then you may put a land card from your hand onto the battlefield.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::PutLandFromHandOntoBattlefield { tapped: false },
                ]),
                intervening_if: Some(Condition::OpponentHasPoisonCounters(3)),
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        // PB-DX4 (2026-08-01, OOS-DP10-8): Complete (by the `#[default]` derive) -> partial.
        //
        // MCP-verified printed text: "Corrupted — At the beginning of your end step, if an
        // opponent has three or more poison counters, draw a card, then YOU MAY put a land card
        // from your hand onto the battlefield."
        //
        // The Corrupted trigger above authors that as an unconditional
        // `Sequence(DrawCards, PutLandFromHandOntoBattlefield)`, so the controller is FORCED to
        // put a land out of hand every qualifying end step -- giving up card advantage,
        // information, and any land they were holding for a later turn.
        //
        // Same class as OOS-DP10-8's Smuggler's Copter and PB-DX3b's `emeria_the_sky_ruin`: a
        // COSTLESS "you may" has no DSL representation (`MayPayThenEffect` requires a `Cost`
        // and a free one always trivially pays; `MayPayOrElse` and `Effect::Choose` are both
        // barred from `Complete` by `effect_choose_gate.rs`; PB-DP9's `pending_effect_choice`
        // channel serves search/scry/surveil only). `partial` rather than `known_wrong`
        // because the trigger, its CR 603.4 intervening-if and the draw are all correct --
        // only the optionality of the land-put is lost.
        completeness: Completeness::partial(
            "Printed 'then you MAY put a land card from your hand onto the battlefield' is \
             authored as an unconditional Sequence(DrawCards, PutLandFromHandOntoBattlefield), \
             forcing the land-put every qualifying end step. A costless 'you may' has no DSL \
             representation (audit §5 DP-12; same class as Smuggler's Copter and \
             emeria_the_sky_ruin). The trigger, its intervening-if and the draw are correct.",
        ),
        ..Default::default()
    }
}

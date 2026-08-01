// 67. Smuggler's Copter — {2}, Artifact — Vehicle 3/3; Flying; Crew 1;
// Whenever it attacks or blocks, you may draw a card. If you do, discard a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("smugglers-copter"),
        name: "Smuggler's Copter".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact].iter().copied().collect(),
            subtypes: ["Vehicle".to_string()]
                .iter()
                .cloned()
                .map(SubType)
                .collect(),
            ..Default::default()
        },
        oracle_text: "Flying\nCrew 1 (Tap any number of creatures you control with total power 1 \
                      or more: This Vehicle becomes an artifact creature until end of \
                      turn.)\nWhenever Smuggler's Copter attacks or blocks, you may draw a card. \
                      If you do, discard a card."
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Crew(1)),
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::DiscardCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenBlocks,
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::DiscardCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
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
        // PB-DX4 (2026-08-01, OOS-DP10-8): Complete -> known_wrong.
        //
        // MCP-verified printed text: "Whenever this Vehicle attacks or blocks, YOU MAY draw a
        // card. If you do, discard a card." Both triggers above author that as an
        // UNCONDITIONAL `Effect::Sequence(vec![DrawCards, DiscardCards])`, so the controller is
        // FORCED to loot on every attack and every block -- and on an empty library the forced
        // draw loses the game outright (CR 704.5b).
        //
        // This is the 20th instance of the class audit §5's DP-12 row already owns: a COSTLESS
        // "you may" on a trigger has no DSL representation (`MayPayThenEffect` requires a
        // `Cost`, and a free one always trivially pays; `MayPayOrElse` and `Effect::Choose` are
        // both barred from `Complete` by `effect_choose_gate.rs`; PB-DP9's
        // `pending_effect_choice` channel serves search/scry/surveil only). The other 19
        // instances are already marked `known_wrong`; this def was simply left `Complete`, so
        // the marker -- not the encoding -- is what is wrong here.
        completeness: Completeness::known_wrong(
            "Printed 'you MAY draw a card. If you do, discard a card' is authored as an \
             unconditional Sequence(DrawCards, DiscardCards) on both the attack and block \
             triggers: the controller is forced to loot every attack and block, and decks out on \
             an empty library. A costless 'you may' on a trigger has no DSL representation (audit \
             §5 DP-12; the other 19 instances of this class are already known_wrong). Closing it \
             needs the DP-12 owning engine PB, not a card-def edit.",
        ),
    }
}

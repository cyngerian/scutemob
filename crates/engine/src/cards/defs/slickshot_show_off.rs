// Slickshot Show-Off — {1}{R}, Creature — Bird Wizard 1/2
// Flying, Haste
// Whenever you cast a noncreature spell, gets +2/+0 until end of turn.
// Plot {1}{R} (CR 702.170)
//
// DSL gap: WheneverYouCastSpell has no spell-type filter field, so the
// "noncreature spell" condition on the pump trigger cannot be expressed.
// That triggered ability is omitted pending a DSL extension adding a
// `spell_type_filter` field (or a dedicated TriggerCondition variant).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("slickshot-show-off"),
        name: "Slickshot Show-Off".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Bird", "Wizard"]),
        oracle_text: "Flying, haste\nWhenever you cast a noncreature spell, this creature gets +2/+0 until end of turn.\nPlot {1}{R} (You may pay {1}{R} and exile this card from your hand. Cast it as a sorcery on a later turn without paying its mana cost. Plot only as a sorcery.)".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // Whenever you cast a noncreature spell, gets +2/+0 until end of turn.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: None,
                    noncreature_only: true,
                },
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPower(2),
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Plot),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Plot,
                cost: ManaCost { generic: 1, red: 1, ..Default::default() },
                details: None,
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
    }
}

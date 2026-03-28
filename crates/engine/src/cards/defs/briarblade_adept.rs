// Briarblade Adept — {4}{B}, Creature — Elf Assassin 3/4; Encore {3}{B}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("briarblade-adept"),
        name: "Briarblade Adept".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Assassin"]),
        oracle_text: "Whenever this creature attacks, target creature an opponent controls gets -1/-1 until end of turn.\nEncore {3}{B} ({3}{B}, Exile this card from your graveyard: For each opponent, create a token copy that attacks that opponent this turn if able. They gain haste. Sacrifice them at the beginning of the next end step. Activate only as a sorcery.)".to_string(),
        abilities: vec![
            // Whenever this creature attacks, target creature an opponent controls gets -1/-1 until EOT.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: crate::state::EffectLayer::PtModify,
                        modification: crate::state::LayerModification::ModifyBoth(-1),
                        filter: crate::state::EffectFilter::DeclaredTarget { index: 0 },
                        duration: crate::state::EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Encore),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Encore,
                cost: ManaCost { generic: 3, black: 1, ..Default::default() },
                details: None,
            },
        ],
        power: Some(3),
        toughness: Some(4),
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

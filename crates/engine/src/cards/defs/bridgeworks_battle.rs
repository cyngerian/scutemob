// Bridgeworks Battle // Tanglespan Bridgeworks
// {2}{G} Sorcery: Target creature you control gets +2/+2 until end of turn.
// It fights up to one target creature you don't control.
// (Back face: Tanglespan Bridgeworks — Land)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bridgeworks-battle"),
        name: "Bridgeworks Battle // Tanglespan Bridgeworks".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Target creature you control gets +2/+2 until end of turn. It fights up to one target creature you don't control. (Each deals damage equal to its power to the other.)".to_string(),
        abilities: vec![
            // CR 601.2c: Spell targets — creature you control (index 0) must be chosen;
            // creature you don't control (index 1) is "up to one" (optional target).
            // TODO: "up to one" optional targeting is not yet supported in the DSL —
            // using mandatory second target as approximation (requires both targets to cast).
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // +2/+2 until end of turn to the creature you control.
                    // CR 611.3a: Until-end-of-turn continuous effect via Layer 7c.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyBoth(2),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    // CR 701.14a: It fights up to one target creature you don't control.
                    // CR 701.14b: If index 1 target is gone, no damage dealt (handled by Fight).
                    Effect::Fight {
                        attacker: EffectTarget::DeclaredTarget { index: 0 },
                        defender: EffectTarget::DeclaredTarget { index: 1 },
                    },
                ]),
                targets: vec![
                    TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                        controller: TargetController::Opponent,
                        ..Default::default()
                    }),
                ],
                modes: None,
                cant_be_countered: false,
            },
        ],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Tanglespan Bridgeworks".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "Tanglespan Bridgeworks enters tapped.\n{T}: Add {G}.".to_string(),
            power: None,
            toughness: None,
            abilities: vec![
                // CR 614.1c: enters-tapped self-replacement.
                AbilityDefinition::Replacement {
                    trigger: ReplacementTrigger::WouldEnterBattlefield {
                        filter: ObjectFilter::Any,
                    },
                    modification: ReplacementModification::EntersTapped,
                    is_self: true,
                    unless_condition: None,
                },
                // {T}: Add {G}.
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 0, 0, 1, 0),
                    },
                    timing_restriction: None,
                    targets: vec![],
                activation_condition: None,
                activation_zone: None,
                },
            ],
            color_indicator: None,
        }),
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
    }
}

// Revitalizing Repast // Old-Growth Grove — Modal DFC (The Brothers' War)
// Front: {B/G} Instant — Put a +1/+1 counter on target creature.
//        It gains indestructible until end of turn.
// Back:  Old-Growth Grove — Land, enters tapped.
//        {T}: Add {G}.
//
// CR 712.8a: While outside the battlefield or in a zone other than battlefield/stack,
//   the card has only front-face characteristics.
// CR 712.8e: Back face mana value uses front face's mana cost.
// CR 614.1c: Enters-tapped replacement effect.
// CR 702.12a: Indestructible — continuous effect until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("revitalizing-repast"),
        name: "Revitalizing Repast".to_string(),
        // Front face: {B/G} hybrid instant
        mana_cost: Some(ManaCost {
            hybrid: vec![HybridMana::ColorColor(ManaColor::Black, ManaColor::Green)],
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Put a +1/+1 counter on target creature. It gains indestructible until end of turn.".to_string(),
        abilities: vec![
            // CR 702.12a + CR 611.3a: +1/+1 counter + indestructible until EOT on target creature.
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeywords(
                                [KeywordAbility::Indestructible].into_iter().collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                        }),
                    },
                ]),
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },
        ],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Old-Growth Grove".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "Old-Growth Grove enters tapped.\n{T}: Add {G}.".to_string(),
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
                },
            ],
            color_indicator: None,
        }),
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
    }
}

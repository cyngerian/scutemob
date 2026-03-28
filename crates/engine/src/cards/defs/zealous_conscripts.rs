// Zealous Conscripts — {4}{R}, Creature — Human Warrior 3/3
// Haste; ETB: gain control of target permanent until end of turn, untap it, give it haste
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("zealous-conscripts"),
        name: "Zealous Conscripts".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Warrior"]),
        oracle_text: "Haste\nWhen this creature enters, gain control of target permanent until end of turn. Untap that permanent. It gains haste until end of turn.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // CR 613.1b: ETB — gain control of target permanent until end of turn,
            // untap it, it gains haste until end of turn.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    Effect::GainControl {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        duration: EffectDuration::UntilEndOfTurn,
                    },
                    Effect::UntapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeywords(
                                [KeywordAbility::Haste].into_iter().collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPermanent],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}

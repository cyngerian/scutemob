// Nezumi Prowler — {1}{B}, Artifact Creature — Rat Ninja 3/1
// Ninjutsu {1}{B}; ETB: target creature you control gains deathtouch and lifelink until EOT.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nezumi-prowler"),
        name: "Nezumi Prowler".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Rat", "Ninja"]),
        oracle_text: "Ninjutsu {1}{B} ({1}{B}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)\nWhen this creature enters, target creature you control gains deathtouch and lifelink until end of turn.".to_string(),
        power: Some(3),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { generic: 1, black: 1, ..Default::default() },
            },
            // CR 603.1: When this creature enters, target creature you control gains
            // deathtouch and lifelink until end of turn.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Deathtouch),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Lifelink),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                })],
            },
        ],
        ..Default::default()
    }
}

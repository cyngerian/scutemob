// Ezuri, Renegade Leader — {1}{G}{G}, Legendary Creature — Elf Warrior 2/2
// {G}: Regenerate another target Elf.
// {2}{G}{G}{G}: Elf creatures you control get +3/+3 and gain trample until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ezuri-renegade-leader"),
        name: "Ezuri, Renegade Leader".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Warrior"],
        ),
        oracle_text: "{G}: Regenerate another target Elf.\n{2}{G}{G}{G}: Elf creatures you control get +3/+3 and gain trample until end of turn.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // {G}: Regenerate another target Elf.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { green: 1, ..Default::default() }),
                effect: Effect::Regenerate {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: None,
                // NOTE: oracle says "another target Elf" — self-exclusion on TargetRequirement
                // not in DSL; implemented without self-exclusion.
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    has_subtype: Some(SubType("Elf".to_string())),
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // CR 613.4c / CR 613.1f: "{2}{G}{G}{G}: Elf creatures you control get +3/+3
            // and gain trample until end of turn." Uses CreaturesYouControlWithSubtype
            // (includes source Ezuri himself, as oracle says "Elf creatures you control").
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 2, green: 3, ..Default::default() }),
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyBoth(3),
                            filter: EffectFilter::CreaturesYouControlWithSubtype(
                                SubType("Elf".to_string()),
                            ),
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Trample),
                            filter: EffectFilter::CreaturesYouControlWithSubtype(
                                SubType("Elf".to_string()),
                            ),
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

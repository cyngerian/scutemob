// Access Denied — {3}{U}{U}, Instant
// Counter target spell. Create X 1/1 colorless Thopter artifact creature tokens with
// flying, where X is that spell's mana value.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("access-denied"),
        name: "Access Denied".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            blue: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell. Create X 1/1 colorless Thopter artifact creature \
                      tokens with flying, where X is that spell's mana value."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 701.5: Counter target spell, then create X 1/1 Thopter tokens where X is
            // that spell's mana value. EffectAmount::ManaValueOf reads the countered spell's
            // MV via its fizzle snapshot (works after CounterSpell has moved it off the
            // stack — see feed_the_swarm.rs for the same read-after-move pattern).
            effect: Effect::Sequence(vec![
                Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    exile_instead: false,
                },
                Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Thopter".to_string(),
                        card_types: [CardType::Artifact, CardType::Creature]
                            .into_iter()
                            .collect(),
                        subtypes: [SubType("Thopter".to_string())].into_iter().collect(),
                        colors: imbl::OrdSet::new(),
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::ManaValueOf(EffectTarget::DeclaredTarget { index: 0 }),
                        supertypes: imbl::OrdSet::new(),
                        keywords: [KeywordAbility::Flying].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
            ]),
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}

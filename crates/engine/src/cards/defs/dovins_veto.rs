// Dovin's Veto — {W}{U}, Instant
// This spell can't be countered.
// Counter target noncreature spell.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dovins-veto"),
        name: "Dovin's Veto".to_string(),
        mana_cost: Some(ManaCost { white: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "This spell can't be countered.\nCounter target noncreature spell.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter {
                    non_creature: true,
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: true,
            },
        ],
        ..Default::default()
    }
}

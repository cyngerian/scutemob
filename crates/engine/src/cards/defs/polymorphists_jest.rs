// Polymorphist's Jest — {1}{U}{U}, Instant
// Until end of turn, each creature target player controls loses all abilities and
// becomes a blue Frog with base power and toughness 1/1.
//
// Layers 4/5/6/7b: SetTypeLine, SetColors, RemoveAllAbilities, SetPt.
// TODO: Targets "target player" — no TargetRequirement::TargetPlayer.
// Approximated as affecting all creatures via ForEach or as Effect::Nothing.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("polymorphists-jest"),
        name: "Polymorphist's Jest".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Until end of turn, each creature target player controls loses all abilities and becomes a blue Frog with base power and toughness 1/1.".to_string(),
        abilities: vec![
            // TODO: "target player" targeting, then apply to all creatures that player controls.
            // Needs TargetRequirement::TargetPlayer + EffectFilter::CreaturesControlledByTarget.
            // Leaving as Effect::Nothing — correct targeting deferred to M10.
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

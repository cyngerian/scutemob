// 25. Doom Blade — {1B}, Instant; destroy target non-black creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("doom-blade"),
        name: "Doom Blade".to_string(),
        mana_cost: Some(ManaCost { black: 1, generic: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target non-black creature.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                exclude_colors: Some([Color::Black].into_iter().collect()),
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

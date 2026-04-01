// Snuff Out — {3}{B}, Instant
// If you control a Swamp, you may pay 4 life rather than pay this spell's mana cost.
// Destroy target nonblack creature. It can't be regenerated.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("snuff-out"),
        name: "Snuff Out".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If you control a Swamp, you may pay 4 life rather than pay this spell's mana cost.\nDestroy target nonblack creature. It can't be regenerated.".to_string(),
        abilities: vec![
            // TODO: Land-conditional life-payment alt cost not in DSL.
            AbilityDefinition::Spell {
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: true,
                },
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    exclude_colors: Some([Color::Black].into_iter().collect()),
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

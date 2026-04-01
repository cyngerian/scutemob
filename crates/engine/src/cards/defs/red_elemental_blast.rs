// Red Elemental Blast — {R}, Instant
// Choose one — Counter target blue spell. / Destroy target blue permanent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("red-elemental-blast"),
        name: "Red Elemental Blast".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Counter target blue spell.\n• Destroy target blue permanent.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            targets: vec![
                // Mode 0: target blue spell
                TargetRequirement::TargetSpellWithFilter(TargetFilter {
                    colors: Some([Color::Blue].into_iter().collect()),
                    ..Default::default()
                }),
                // Mode 1: target blue permanent
                TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    colors: Some([Color::Blue].into_iter().collect()),
                    ..Default::default()
                }),
            ],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Counter target blue spell.
                    Effect::CounterSpell {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    // Mode 1: Destroy target blue permanent.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                        cant_be_regenerated: false,
                    },
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

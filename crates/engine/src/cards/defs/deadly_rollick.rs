// Deadly Rollick — {3}{B}, Instant
// If you control a commander, you may cast this spell without paying its mana cost.
// Exile target creature.
// TODO: Commander-conditional free cast alt cost — no DSL primitive for
//   "if you control a commander, may cast without paying mana cost"
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("deadly-rollick"),
        name: "Deadly Rollick".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If you control a commander, you may cast this spell without paying its mana cost.\nExile target creature.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::ExileObject {
                target: EffectTarget::DeclaredTarget { index: 0 },
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

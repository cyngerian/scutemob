// Deadly Rollick — {3}{B}, Instant
// If you control a commander, you may cast this spell without paying its mana cost.
// Exile target creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("deadly-rollick"),
        name: "Deadly Rollick".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If you control a commander, you may cast this spell without paying its mana cost.\nExile target creature.".to_string(),
        abilities: vec![
            // CR 118.9 / 2020-04-17 ruling: cast without paying mana cost if you control
            // any commander on the battlefield (any player's commander qualifies).
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::CommanderFreeCast,
                cost: ManaCost::default(),
                details: None,
            },
            AbilityDefinition::Spell {
                effect: Effect::ExileObject {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

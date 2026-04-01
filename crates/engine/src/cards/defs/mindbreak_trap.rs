// Mindbreak Trap — {2}{U}{U}, Instant — Trap
// If an opponent cast three or more spells this turn, you may pay {0} rather
// than pay this spell's mana cost.
// Exile any number of target spells.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mindbreak-trap"),
        name: "Mindbreak Trap".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If an opponent cast three or more spells this turn, you may pay {0} rather than pay this spell's mana cost.\nExile any number of target spells.".to_string(),
        abilities: vec![
            // TODO: Conditional free-cast (opponent cast 3+ spells) not in DSL.
            // TODO: "any number of target spells" — variable targets not supported.
            AbilityDefinition::Spell {
                effect: Effect::ExileObject {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetSpell],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

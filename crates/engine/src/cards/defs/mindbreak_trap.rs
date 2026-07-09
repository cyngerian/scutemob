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
            // ENGINE-BLOCKED: (1) Trap alternative cost — no AltCostKind::Trap, so the
            // "if an opponent cast three or more spells this turn, you may pay {0}" alt-cost
            // cannot be paid. (2) "any number of target spells" — variable target counts are
            // not supported.
            // (The condition itself is now available as Condition::OpponentCastNSpells(3) —
            // PB-AC6. It is the alt-cost wrapper that is missing, not the count.)
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

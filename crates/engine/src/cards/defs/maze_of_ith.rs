// Maze of Ith — Land
// {T}: Untap target attacking creature. Prevent all combat damage that would be dealt to and dealt by that creature this turn.
//
// DSL gap: "Prevent all combat damage dealt to and by that creature this turn" requires
//   a damage-prevention continuous effect on the untapped creature (no PreventDamage effect).
// Implementing UntapPermanent only; prevention effect is a TODO.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("maze-of-ith"),
        name: "Maze of Ith".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Untap target attacking creature. Prevent all combat damage that would be dealt to and dealt by that creature this turn.".to_string(),
        abilities: vec![
            // TODO: DSL gap — "target attacking creature" requires TargetRequirement::TargetAttackingCreature
            // (does not exist). Untap + prevent combat damage also needs PreventCombatDamage effect.
            // Stripped per W5 policy — targeting any creature instead of attacking creature is wrong game state.
        ],
        ..Default::default()
    }
}

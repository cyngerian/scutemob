// Dragonstorm Globe — {3}, Artifact
// Each Dragon you control enters with an additional +1/+1 counter on it.
// {T}: Add one mana of any color.
//
// TODO: "Each Dragon you control enters with an additional +1/+1 counter" — ETB replacement
//   effect filtered by subtype not expressible in DSL. Implementing only the mana ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragonstorm-globe"),
        name: "Dragonstorm Globe".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Each Dragon you control enters with an additional +1/+1 counter on it.\n{T}: Add one mana of any color.".to_string(),
        abilities: vec![
            // TODO: ETB replacement for Dragons (+1/+1 counter)
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}

// Replicating Ring — {3}, Snow Artifact
// {T}: Add one mana of any color.
// At the beginning of your upkeep, put a night counter on this.
//   Then if 8+, remove all and create 8 Replicated Ring tokens.
//
// TODO: Upkeep trigger with counter threshold + token creation of named artifact tokens
//   with mana abilities. Implementing only the mana tap ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("replicating-ring"),
        name: "Replicating Ring".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: supertypes(&[SuperType::Snow], &[CardType::Artifact]),
        oracle_text: "{T}: Add one mana of any color.\nAt the beginning of your upkeep, put a night counter on Replicating Ring. Then if it has eight or more night counters on it, remove all of them and create eight colorless snow artifact tokens named Replicated Ring with \"{T}: Add one mana of any color.\"".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // TODO: upkeep night counter trigger + 8-counter token creation
        ],
        ..Default::default()
    }
}

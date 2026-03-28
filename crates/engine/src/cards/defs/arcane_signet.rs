// 2. Arcane Signet — {2}, Artifact, tap: add one mana of any color in your
// commander's color identity. Modelled as AddManaAnyColor (simplified).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("arcane-signet"),
        name: "Arcane Signet".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Add one mana of any color in your commander's color identity."
            .to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
            timing_restriction: None,
            targets: vec![],
                activation_condition: None,
                activation_zone: None,
        }],
        ..Default::default()
    }
}

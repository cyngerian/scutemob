// 3. Commander's Sphere — {3}, Artifact, tap: add one mana of any color;
// sacrifice: draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("commanders-sphere"),
        name: "Commander's Sphere".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Add one mana of any color in your commander's color \
                      identity.\nSacrifice Commander's Sphere: Draw a card."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::SacrificeSelf,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::known_wrong(
            "PB-EF12 (EF-W-PB2-3) fixed the colour-choice stub; see command_tower.rs / \
             arcane_signet.rs for the identical remaining blocker: the choice is unrestricted \
             (all five colours) instead of restricted to the controller's commander's color \
             identity, which the engine has no runtime mechanism to enforce. Filed as OOS-EF12-1.",
        ),
        ..Default::default()
    }
}

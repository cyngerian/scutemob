// Mox Amber — {T}: Add one mana of any color among legendary creatures and planeswalkers you c
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mox-amber"),
        name: "Mox Amber".to_string(),
        mana_cost: Some(ManaCost {
            ..Default::default()
        }),
        types: full_types(&[SuperType::Legendary], &[CardType::Artifact], &[]),
        oracle_text: "{T}: Add one mana of any color among legendary creatures and planeswalkers \
                      you control."
            .to_string(),
        abilities: vec![AbilityDefinition::Activated {
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
        }],
        completeness: Completeness::known_wrong(
            "PB-EF12 (EF-W-PB2-3) fixed the colour-choice stub — `any_color: true` mana abilities \
             now resolve to a real chosen colour instead of ManaColor::Colorless (CR \
             111.10a/605.3b). Real remaining blocker: the choice must be restricted to the set of \
             colors among legendary creatures and planeswalkers the controller controls, which \
             the engine cannot compute at mana-activation time (same class of gap as \
             command_tower.rs's commander-color-identity restriction, just a different colour \
             set). Filed as OOS-EF12-1.",
        ),
        ..Default::default()
    }
}

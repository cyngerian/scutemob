// 9. Command Tower — Land, tap: add one mana of any color in your commander's
// color identity.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("command-tower"),
        name: "Command Tower".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add one mana of any color in your commander's color identity."
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
             111.10a/605.3b). Real remaining blocker (unchanged, not fixed by that PB, and NOT \
             the same defect as the stub — the original known_wrong note here undercounted this \
             card's blockers): the choice is offered from all five colours, unrestricted, when it \
             should be restricted to the controller's commander's color identity — there is no \
             engine mechanism that restricts an any_color mana ability's option set to a computed \
             colour subset (compute_color_identity exists only for deck-build validation, not \
             runtime mana legality; see arcane_signet.rs, the same defect). Filed as OOS-EF12-1.",
        ),
        ..Default::default()
    }
}

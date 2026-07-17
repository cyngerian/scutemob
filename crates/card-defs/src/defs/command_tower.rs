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
        }],
        completeness: Completeness::known_wrong(
            "SF-11 (CR 106.1a/106.1b): this card's \"any color\" mana resolves to one COLORLESS \
             mana (Effect::AddManaAnyColor family; effects/mod.rs and handle_tap_for_mana step 8 \
             both add ManaColor::Colorless). Colorless is a mana TYPE, not a color, so {C} is \
             outside the option set \"any color\" offers — wrong game state, not an omitted \
             clause. Un-mark when a color channel for any-color mana lands (SR-37 / scutemob-93).",
        ),
        ..Default::default()
    }
}

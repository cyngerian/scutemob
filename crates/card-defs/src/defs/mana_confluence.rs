// Mana Confluence — Land, {T}, Pay 1 life: Add one mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mana-confluence"),
        name: "Mana Confluence".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}, Pay 1 life: Add one mana of any color.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sequence(vec![Cost::Tap, Cost::PayLife(1)]),
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
            "CR 106.1b: '{T}, Pay 1 life: Add one mana of any color' adds one COLORLESS mana. \
             Colorless is a mana TYPE, not a color (CR 106.1a/106.1b), so {C} is not one of the \
             legal options 'any color' offers — this is wrong state, not an omitted clause. SR-34 \
             correctly lowered it to a real ManaAbility and its life cost IS paid (probed: life \
             40 -> 39, stack empty), but handle_tap_for_mana step 8 reads `any_color` and adds \
             ManaColor::Colorless, exactly as Effect::AddManaAnyColor does in effects/mod.rs — \
             escaping into a ManaAbility does not help. Blocked on interactive/deterministic \
             color choice for any_color mana.",
        ),
        ..Default::default()
    }
}

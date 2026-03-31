// Demolition Field — Land.
// "{T}: Add {C}."
// "{2}, {T}, Sacrifice this land: Destroy target nonbasic land an opponent
// controls. That land's controller may search their library for a basic land
// card, put it onto the battlefield, then shuffle. You may search your library
// for a basic land card, put it onto the battlefield, then shuffle."
//
// The {T}: Add {C} ability is fully expressible.
// The activated ability has two DSL gaps:
//   1. "target nonbasic land" — TargetFilter has no "non_basic: bool" field.
//      Using has_card_type: Land allows targeting basic lands too (wrong game state).
//   2. "That land's controller may search their library" — searching with
//      PlayerTarget for the controller of a destroyed permanent (ControllerOf
//      a destroyed object) is not reliably resolvable post-destruction via
//      the current EffectTarget/PlayerTarget model.
// W5: the activated ability is omitted — partial implementation produces wrong game state.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("demolition-field"),
        name: "Demolition Field".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{2}, {T}, Sacrifice this land: Destroy target nonbasic land an opponent controls. That land's controller may search their library for a basic land card, put it onto the battlefield, then shuffle. You may search your library for a basic land card, put it onto the battlefield, then shuffle.".to_string(),
        abilities: vec![
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // TODO: {2}, {T}, Sacrifice this land: Destroy target nonbasic land an opponent controls.
            // That land's controller may search their library for a basic land card,
            // put it onto the battlefield, then shuffle.
            // You may search your library for a basic land card, put it onto the battlefield,
            // then shuffle.
            // Blocked by: (1) no TargetFilter::non_basic field; (2) "that land's controller"
            // requires PlayerTarget::ControllerOf a destroyed permanent (post-destruction LKI).
        ],
        ..Default::default()
    }
}

// City of Brass — Land
// "Whenever City of Brass becomes tapped, it deals 1 damage to you."
// "{T}: Add one mana of any color."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("city-of-brass"),
        name: "City of Brass".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "Whenever City of Brass becomes tapped, it deals 1 damage to you.\n{T}: Add \
                      one mana of any color."
            .to_string(),
        abilities: vec![
            // Triggered: whenever this becomes tapped (any source), deal 1 damage to controller.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenSelfBecomesTapped,
                effect: Effect::DealDamage {
                    source: None,
                    target: EffectTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // Mana ability: {T}: Add one mana of any color.
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
        ],
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

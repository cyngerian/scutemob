// Ancient Tomb — Land.
// "{T}: Add {C}{C}. Ancient Tomb deals 2 damage to you."
// CR 605.1a: this IS a mana ability — it produces mana, has no target, and is not a
// loyalty ability; the damage rider does not change that. It is authored as an
// `Activated { Cost::Tap, Sequence([AddMana, DealDamage{Controller}]) }` because that is
// the shape `try_as_tap_mana_ability` recognises as a pain land: it registers a real
// `ManaAbility { produces: {Colorless: 2}, damage_to_controller: 2 }`, so it does not use
// the stack and is reachable via `Command::TapForMana`. Same pattern as caves_of_koilos.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ancient-tomb"),
        name: "Ancient Tomb".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}{C}. Ancient Tomb deals 2 damage to you.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Sequence(vec![
                    Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 0, 0, 0, 2),
                    },
                    Effect::DealDamage {
                        target: EffectTarget::Controller,
                        amount: EffectAmount::Fixed(2),
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

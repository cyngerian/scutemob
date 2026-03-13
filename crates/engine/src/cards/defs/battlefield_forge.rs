// Battlefield Forge — Land; {T}: Add {C}; {T}: Add {R} or {W} (deals 1 damage to you).
// TODO: the pain-land damage clause ("This land deals 1 damage to you") for the
// second ability requires a conditional self-damage effect not yet in DSL.
// Implementing colorless tap and choose {R}/{W} without the damage.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("battlefield-forge"),
        name: "Battlefield Forge".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}: Add {R} or {W}. This land deals 1 damage to you.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {T}: Add {R} or {W}. This land deals 1 damage to you.
            // DSL gap: no self-damage side effect on mana abilities.
        ],
        ..Default::default()
    }
}

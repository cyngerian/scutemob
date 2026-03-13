// Shivan Reef — Painland, {T}: Add {C}. {T}: Add {U} or {R} (deals 1 damage to you, TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shivan-reef"),
        name: "Shivan Reef".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}: Add {U} or {R}. This land deals 1 damage to you.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {T}: Add {U} or {R} (deals 1 damage) — Sequence effect combining AddMana + DealDamage with choice not in DSL
        ],
        ..Default::default()
    }
}

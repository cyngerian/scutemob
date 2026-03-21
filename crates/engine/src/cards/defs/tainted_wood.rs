// Tainted Wood — Land
// {T}: Add {C}. {T}: Add {B} or {G}, only if you control a Swamp (conditional, not expressible).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tainted-wood"),
        name: "Tainted Wood".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}: Add {B} or {G}. Activate only if you control a Swamp.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // TODO: {T}: Add {B} or {G}, Activate only if you control a Swamp —
            // conditional activation restriction (requires controlling a subtype
            // land) is not expressible in the DSL (no Cost::IfControlsSubtype
            // variant or activation condition)
        ],
        ..Default::default()
    }
}

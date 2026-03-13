// Tainted Field — Land
// {T}: Add {C}. {T}: Add {W} or {B}, only if you control a Swamp (conditional, not expressible).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tainted-field"),
        name: "Tainted Field".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}: Add {W} or {B}. Activate only if you control a Swamp.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {T}: Add {W} or {B}, Activate only if you control a Swamp —
            // conditional activation restriction (requires controlling a subtype
            // land) is not expressible in the DSL (no Cost::IfControlsSubtype
            // variant or activation condition)
        ],
        ..Default::default()
    }
}

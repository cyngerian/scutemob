// Flooded Grove — Land; {T}: Add {C}; {G/U}, {T}: Add {G}{G}, {G}{U}, or {U}{U}.
// TODO: the filter ability ({G/U}, {T}: Add {G}{G}, {G}{U}, or {U}{U}) requires
// hybrid mana costs and a 3-way choice — not expressible in current DSL.
// Implementing colorless tap only.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flooded-grove"),
        name: "Flooded Grove".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{G/U}, {T}: Add {G}{G}, {G}{U}, or {U}{U}.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: {G/U}, {T}: Add {G}{G}, {G}{U}, or {U}{U} — hybrid mana cost and
            // 3-way double-mana choice not expressible in current DSL.
        ],
        ..Default::default()
    }
}

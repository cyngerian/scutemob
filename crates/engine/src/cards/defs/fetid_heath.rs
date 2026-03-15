// Fetid Heath — Land (filterland)
// {T}: Add {C}. {W/B},{T}: Add {W}{W}, {W}{B}, or {B}{B}.
// The hybrid mana cost and triple-choice output are partially expressible;
// implementing the three color-mana outputs as a Choose (ignoring the
// {W/B} cost requirement — TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fetid-heath"),
        name: "Fetid Heath".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{W/B}, {T}: Add {W}{W}, {W}{B}, or {B}{B}.".to_string(),
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
            // {W/B}, {T}: Add {W}{W}, {W}{B}, or {B}{B}
            // TODO: Triple-choice mana output not expressible; defaulting to {W}{B}.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        hybrid: vec![HybridMana::ColorColor(ManaColor::White, ManaColor::Black)],
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}

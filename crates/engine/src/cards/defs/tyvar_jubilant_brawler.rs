// Tyvar, Jubilant Brawler — {1}{B}{G}, Legendary Planeswalker — Tyvar (loyalty 3)
// You may activate abilities of creatures you control as though those creatures had haste.
// +1: Untap up to one target creature.
// -2: Mill three cards, then you may return a creature card with mana value 2 or less from your graveyard to the battlefield.
// TODO: DSL gap — planeswalker loyalty abilities not supported in the current DSL.
// The static haste-activation ability, +1 untap trigger, and -2 mill+reanimate are all beyond
// current DSL coverage (no PlaneswalkerLoyalty ability type, no ActivateAsIfHaste static effect,
// no return_from_graveyard effect with mana value filter).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tyvar-jubilant-brawler"),
        name: "Tyvar, Jubilant Brawler".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Tyvar"],
        ),
        oracle_text: "You may activate abilities of creatures you control as though those creatures had haste.\n+1: Untap up to one target creature.\n\u{2212}2: Mill three cards, then you may return a creature card with mana value 2 or less from your graveyard to the battlefield.".to_string(),
        abilities: vec![
            // TODO: static — creatures you control can activate abilities as though they had haste
            // DSL gap: no ActivateAsIfHaste static continuous effect.
            // TODO: +1 loyalty ability — untap up to one target creature
            // TODO: -2 loyalty ability — mill 3, then optionally return creature card MV<=2 from GY to BF
            // DSL gap: planeswalker loyalty abilities not supported; no return_from_graveyard effect.
        ],
        ..Default::default()
    }
}

// Tyvar, Jubilant Brawler — {1}{B}{G}, Legendary Planeswalker — Tyvar (loyalty 3)
// You may activate abilities of creatures you control as though those creatures had haste.
// +1: Untap up to one target creature.
// −2: Mill three cards, then you may return a creature card with mana value 2 or less
//     from your graveyard to the battlefield.
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
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                // +1: Untap up to one target creature.
                // TODO: "up to one" targeting is optional — needs TargetRequirement::UpToOne variant.
                // For now, uses a simple untap effect with no target.
                effect: Effect::Sequence(vec![]),
                targets: vec![],
            },
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                // −2: Mill three cards, then optionally return creature MV<=2 from GY to BF.
                // TODO: Mill effect + conditional graveyard return with MV filter.
                effect: Effect::Sequence(vec![]),
                targets: vec![],
            },
        ],
        starting_loyalty: Some(3),
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        ..Default::default()
    }
}

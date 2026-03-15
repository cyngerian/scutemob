// Urza's Saga — Enchantment Land / Saga with 3 chapter abilities
// CR 714: Saga chapter abilities with lore counters, sacrifice after III.
// Chapter I: gains "{T}: Add {C}" — modeled as GainLife placeholder (ability-granting not in DSL)
// Chapter II: gains Construct token creation — placeholder
// Chapter III: search library for artifact with mana cost {0} or {1} — needs SearchFilter
// TODO: Chapter I/II need "this Saga gains [ability]" (continuous effect granting activated ability)
// TODO: Chapter III needs SearchLibrary with artifact CMC filter (PB-17)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("urzas-saga"),
        name: "Urza's Saga".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Enchantment, CardType::Land], &["Urza's", "Saga"]),
        oracle_text: "(As this Saga enters and after your draw step, add a lore counter. Sacrifice after III.)\nI — This Saga gains \"{T}: Add {C}.\"\nII — This Saga gains \"{2}, {T}: Create a 0/0 colorless Construct artifact creature token with 'This token gets +1/+1 for each artifact you control.'\"\nIII — Search your library for an artifact card with mana cost {0} or {1}, put it onto the battlefield, then shuffle.".to_string(),
        abilities: vec![
            // Chapter I: This Saga gains "{T}: Add {C}."
            // TODO: Needs "gains activated ability" continuous effect — placeholder GainLife(0)
            AbilityDefinition::SagaChapter {
                chapter: 1,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(0),
                },
                targets: vec![],
            },
            // Chapter II: This Saga gains construct-token-creation activated ability.
            // TODO: Needs "gains activated ability" continuous effect — placeholder GainLife(0)
            AbilityDefinition::SagaChapter {
                chapter: 2,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(0),
                },
                targets: vec![],
            },
            // Chapter III: Search for artifact with CMC 0 or 1.
            // TODO: Needs SearchLibrary with artifact CMC filter (PB-17)
            AbilityDefinition::SagaChapter {
                chapter: 3,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(0),
                },
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}

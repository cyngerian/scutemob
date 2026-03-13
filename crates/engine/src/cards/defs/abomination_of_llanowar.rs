// Abomination of Llanowar — {1}{B}{G}, Legendary Creature — Elf Horror */*
// Vigilance, Menace; P/T = number of Elves you control + Elf cards in graveyard (CDA)
// TODO: P/T CDA (EffectLayer::PtCda counting Elves on battlefield + in graveyard) not
// expressible in DSL — no CardCount filter for subtypes in graveyard. Deferred.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("abomination-of-llanowar"),
        name: "Abomination of Llanowar".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Horror"],
        ),
        oracle_text: "Vigilance; menace (This creature can't be blocked except by two or more creatures.)\nAbomination of Llanowar's power and toughness are each equal to the number of Elves you control plus the number of Elf cards in your graveyard.".to_string(),
        power: None,   // */* CDA — engine SBA skips None toughness; actual P/T set by layer
        toughness: None,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // TODO: CDA — P/T = number of Elves you control + Elf cards in your graveyard.
            // DSL gap: EffectLayer::PtCda / CardCount lacks cross-zone subtype filter.
        ],
        ..Default::default()
    }
}

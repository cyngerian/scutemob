// Ixhel, Scion of Atraxa — {1}{W}{B}{G}, Legendary Creature — Phyrexian Angel 2/5
// Flying, vigilance, toxic 2; Corrupted end-step triggered ability (complex)
// TODO: Corrupted trigger — "each opponent who has 3+ poison counters exiles top card face down;
// you may look at and play those cards." Requires per-opponent conditional exile with
// play-from-exile tracking. No DSL support for player-scoped triggered effects or
// "you may play exiled cards" with per-card tracking. Deferred.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ixhel-scion-of-atraxa"),
        name: "Ixhel, Scion of Atraxa".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Phyrexian", "Angel"],
        ),
        oracle_text: "Flying, vigilance, toxic 2\nCorrupted — At the beginning of your end step, each opponent who has three or more poison counters exiles the top card of their library face down. You may look at and play those cards for as long as they remain exiled, and you may spend mana as though it were mana of any color to cast those spells.".to_string(),
        power: Some(2),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            AbilityDefinition::Keyword(KeywordAbility::Toxic(2)),
            // TODO: Corrupted end-step trigger — per-opponent conditional exile + play-from-exile.
            // DSL gap: no ForEach over opponents with intervening-if, no play-exiled-card tracking.
        ],
        ..Default::default()
    }
}

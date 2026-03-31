// Dimir Infiltrator — {U}{B}, Creature — Spirit 1/3
// This creature can't be blocked.
// Transmute {1}{U}{B} — search for card with same mana value, reveal, to hand.
// TODO: KeywordAbility::Transmute does not exist in the DSL. The transmute activated
// ability (discard this card + pay mana, search for card with equal mana value) has no
// corresponding Cost or Effect variant. Leaving transmute unimplemented per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dimir-infiltrator"),
        name: "Dimir Infiltrator".to_string(),
        mana_cost: Some(ManaCost { blue: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Spirit"]),
        oracle_text:
            "This creature can't be blocked.\n\
             Transmute {1}{U}{B} ({1}{U}{B}, Discard this card: Search your library for a card \
             with the same mana value as this card, reveal it, put it into your hand, then shuffle. \
             Transmute only as a sorcery.)"
                .to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CantBeBlocked),
            // TODO: Transmute is not implemented in the DSL (no KeywordAbility::Transmute,
            // no mana-value-matching filter for SearchLibrary, no discard-this-card cost).
        ],
        ..Default::default()
    }
}

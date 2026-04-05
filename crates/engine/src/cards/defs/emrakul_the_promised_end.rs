// Emrakul, the Promised End — {13}, Legendary Creature — Eldrazi 13/13
// This spell costs {1} less to cast for each card type among cards in your graveyard.
// Flying, trample, protection from instants
// When you cast this spell, you gain control of target opponent during that player's next turn.
// After that turn, that player takes an extra turn.
// TODO: Protection from instants — Protection(filter) for instant card type not in DSL.
// TODO: Cast trigger — gain-control blocked (PB-A/PB-E); extra turn part expressible
//       via Effect::ExtraTurn (PB-C). Entire trigger blocked until player-control
//       infrastructure exists, since the extra turn fires AFTER the controlled turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("emrakul-the-promised-end"),
        name: "Emrakul, the Promised End".to_string(),
        mana_cost: Some(ManaCost {
            generic: 13,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Eldrazi"],
        ),
        oracle_text: "This spell costs {1} less to cast for each card type among cards in your graveyard.\nWhen you cast this spell, you gain control of target opponent during that player's next turn. After that turn, that player takes an extra turn.\nFlying, trample, protection from instants".to_string(),
        power: Some(13),
        toughness: Some(13),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // CR 702.16a: "Protection from instants" — blocks targeting by instants (DEBT).
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromCardType(CardType::Instant),
            )),
            // TODO: Cast trigger — gain-control (PB-A/PB-E) blocks this entire trigger.
            // Extra turn part is now expressible via Effect::ExtraTurn (PB-C) but cannot
            // be sequenced correctly without the controlled-turn infrastructure.
        ],
        self_cost_reduction: Some(SelfCostReduction::CardTypesInGraveyard),
        ..Default::default()
    }
}

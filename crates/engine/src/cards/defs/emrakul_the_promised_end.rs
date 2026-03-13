// Emrakul, the Promised End — {13}, Legendary Creature — Eldrazi 13/13
// Flying, trample, protection from instants
// Cost reduction based on card types in graveyard; cast trigger to control opponent
// TODO: cost reduction (count card types in graveyard), cast trigger to control opponent, and
//       protection from instants (Protection(filter) for card type) require DSL gaps
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
            // TODO: Protection from instants — Protection(filter) for instant card type not in DSL.
            // TODO: Cost reduction based on distinct card types in graveyard — dynamic cost
            // reduction not in DSL.
            // TODO: Cast trigger granting control of opponent for a turn + extra turn —
            // player control and extra turn effects not in DSL.
        ],
        ..Default::default()
    }
}

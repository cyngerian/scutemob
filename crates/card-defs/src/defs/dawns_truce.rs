// Dawn's Truce — {1}{W}, Instant
// Gift a card
// You and permanents you control gain hexproof until end of turn. If the gift was
// promised, permanents you control also gain indestructible until end of turn.
//
// TODO: Gift mechanic draw + hexproof/indestructible continuous effects complex for DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dawns-truce"),
        name: "Dawn's Truce".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Gift a card (You may promise an opponent a gift as you cast this spell. If \
                      you do, they draw a card before its other effects.)\nYou and permanents you \
                      control gain hexproof until end of turn. If the gift was promised, \
                      permanents you control also gain indestructible until end of turn."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Gift),
            // CR 702.174e: the printed "Gift a card". Added by PB-DX29 — this def was
            // marker-only, and `casting.rs` needs BOTH (it gates on the marker, then
            // `resolution.rs` reads THIS variant's `gift_type` to decide what the chosen
            // player gets), so the gift half was unannounceable and would have resolved
            // to nothing had it been announced.
            AbilityDefinition::Gift {
                gift_type: GiftType::Card,
            },
            // TODO: Hexproof + conditional indestructible not easily expressible.
        ],
        completeness: Completeness::partial(
            "Spell effect unimplemented: hexproof-for-you-and-your-permanents plus the \
             conditional indestructible rider are not expressible together. The GIFT half is \
             now authored (PB-DX29) — the note's former claim that this def 'carries only the \
             Gift keyword marker with no cost AbilityDefinition' is no longer true. Remaining \
             primitives needed: Effect::GrantPlayerProtection (effects/mod.rs), \
             ApplyContinuousEffect + AddKeyword + EffectFilter::ControlledBy, branched on \
             Condition::GiftWasGiven.",
        ),
        ..Default::default()
    }
}

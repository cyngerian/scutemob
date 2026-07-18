// Simian Spirit Guide — {2}{R}, Creature — Ape Spirit 2/2
// Exile this card from your hand: Add {R}.
//
// ENGINE-BLOCKED: `Cost::ExileFromHand { color: Color }` (card_definition.rs:1247) is the
// Force of Will-style pitch alt-cost — it exiles a card of a chosen color from hand as part
// of casting a DIFFERENT spell (the color is validated, the specific card recorded on
// `CastSpell.additional_costs` as `AdditionalCost::ExileFromHand { card }`). It is not a
// self-referential "exile this permanent's own card from hand" activation cost. `Cost` has
// `DiscardSelf` (discard the source from hand — Channel, CR 702.34) but no exile analog
// (e.g. an `ExileSelfFromHand` variant). Without that, this ability has no faithful DSL
// expression. The previous def authored the ability with `Cost::Mana(ManaCost::default())`,
// i.e. a free, repeatable, untapped "Add {R}" — that is worse than nothing (infinite mana),
// so it is omitted per W5 rather than shipped wrong.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("simian-spirit-guide"),
        name: "Simian Spirit Guide".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Ape", "Spirit"]),
        oracle_text: "Exile this card from your hand: Add {R}.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        completeness: Completeness::partial(
            "No Cost variant exists for 'exile this card from your hand' as an activation cost. \
             Cost::ExileFromHand{color} (card_definition.rs:1247) is the unrelated pitch alt-cost \
             mechanic (Force of Will family) — it exiles a card from hand to help pay for casting \
             a DIFFERENT spell, not a self-exile activation cost on the source itself. \
             Cost::DiscardSelf (Channel) is the nearest existing shape but discards rather than \
             exiles. Needs a new Cost::ExileSelfFromHand (or equivalent) + activation_zone: Hand. \
             Left inert rather than shipping the prior def's Cost::Mana(default) 'Add {R}' — that \
             was free, repeatable, infinite mana.",
        ),
        ..Default::default()
    }
}

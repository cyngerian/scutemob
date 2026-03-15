// 60. Golgari Grave-Troll — {4G}, Creature — Troll Skeleton 0/4.
// "This creature enters with a +1/+1 counter on it for each creature card in
// your graveyard. {1}, Remove a +1/+1 counter from this creature: Regenerate
// this creature. Dredge 6."
//
// Simplifications for this milestone:
// - Power is fixed at 0 (real card is 0/0 and tracks counters; counter-based
// P/T requires a continuous layer-3 effect, deferred).
// - ETB counter placement (one per creature card in graveyard) deferred — needs
// a TriggeredEffect that counts graveyard contents at resolution.
// - Regeneration cost deferred — requires AbilityDefinition::Activated with a
// RemoveCounter cost variant that does not yet exist in the DSL.
//
// CR 702.52a: Dredge N — if you would draw a card, you may instead mill N cards
// and return this card from your graveyard to your hand. Functions only while
// this card is in the graveyard. Requires >= N cards in library (CR 702.52b).
//
// Dredge 6 is encoded as KeywordAbility::Dredge(6) — engine handles the
// draw-replacement logic when this card is in its owner's graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("golgari-grave-troll"),
        name: "Golgari Grave-Troll".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 1, ..Default::default() }),
        types: creature_types(&["Troll", "Skeleton"]),
        oracle_text:
            "This creature enters with a +1/+1 counter on it for each creature card in your graveyard.\n\
             {1}, Remove a +1/+1 counter from this creature: Regenerate this creature.\n\
             Dredge 6 (If you would draw a card, you may mill six cards instead. If you do, return this card from your graveyard to your hand.)"
                .to_string(),
        power: Some(0),
        toughness: Some(4),
        abilities: vec![
            // CR 702.52a: Dredge 6 marker — checked in drawing logic to offer the
            // draw-replacement option when this card is in its owner's graveyard.
            AbilityDefinition::Keyword(KeywordAbility::Dredge(6)),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
    }
}

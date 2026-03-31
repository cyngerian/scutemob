// Roiling Regrowth — {2}{G} Instant; sacrifice a land (at resolution, not as cost),
// then search for up to two basic lands and put them onto the battlefield tapped, then shuffle.
//
// TODO: SacrificePermanents has no type filter — sacrificing specifically a land is not
// expressible without incorrectly allowing any permanent to be sacrificed. The ruling
// confirms "Sacrifice a land" happens at resolution, not as an additional cost
// (CR 601.2b does not apply). When SacrificePermanents gains a CardType/subtype filter
// (e.g., filter: Some(TargetFilter { has_card_type: Some(CardType::Land), .. })),
// implement as Effect::Sequence([SacrificePermanents { land filter }, SearchLibrary x2, Shuffle]).
// Per W5 policy, using unfiltered SacrificePermanents produces wrong game state.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("roiling-regrowth"),
        name: "Roiling Regrowth".to_string(),
        mana_cost: Some(ManaCost { green: 1, generic: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Sacrifice a land. Search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

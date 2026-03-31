// Entish Restoration — {2}{G} Instant; sacrifice a land (at resolution), then search
// for up to 2 basic lands to battlefield tapped (or up to 3 if you control a creature
// with power 4 or greater). Then shuffle.
//
// TODO: Two DSL gaps prevent faithful implementation:
//   1. SacrificePermanents has no type filter — sacrificing specifically a land is not
//      expressible without incorrectly allowing any permanent to be sacrificed.
//   2. Condition::YouControlCreatureWithPowerAtLeast(N) does not exist; the conditional
//      branch (search 3 instead of 2 if you control a power-4+ creature) cannot be
//      expressed. A Conditional effect or a new EffectAmount variant would be needed.
// Per W5 policy, a wrong implementation (sacrificing any permanent, ignoring the
// power-4 branch) is worse than an empty one.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("entish-restoration"),
        name: "Entish Restoration".to_string(),
        mana_cost: Some(ManaCost { green: 1, generic: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Sacrifice a land. Search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle. If you control a creature with power 4 or greater, instead search your library for up to three basic land cards, put them onto the battlefield tapped, then shuffle.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

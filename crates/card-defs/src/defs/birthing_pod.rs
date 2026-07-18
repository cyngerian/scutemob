// Birthing Pod — {3}{G/P} Artifact
// {1}{G/P}, {T}, Sacrifice a creature: Search your library for a creature card with mana value equal to
//   1 plus the sacrificed creature's mana value, put that card onto the battlefield, then shuffle.
//   Activate only as a sorcery.
//
// PB-EF10 sweep (2026-07-18): PB-EF10 added EffectAmount::ManaValueOfSacrificedCreature and
// TargetFilter.max_cmc_amount (a runtime UPPER-BOUND cap: "mana value X or less"). Birthing
// Pod needs mana value EQUAL TO 1 + the sacrificed creature's MV, not "or less" — a runtime
// max_cmc_amount alone would wrongly accept any cheaper creature too (legal-but-wrong). This
// needs a runtime EXACT-mana-value filter (or a paired min_cmc_amount set to the same
// EffectAmount) which is out of this PB's declared scope. Still blocked; recorded as a
// follow-up alongside OOS-EF10-1.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("birthing-pod"),
        name: "Birthing Pod".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            phyrexian: vec![PhyrexianMana::Single(ManaColor::Green)],
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "({G/P} can be paid with either {G} or 2 life.)\n{1}{G/P}, {T}, Sacrifice a \
                      creature: Search your library for a creature card with mana value equal to \
                      1 plus the sacrificed creature's mana value, put that card onto the \
                      battlefield, then shuffle. Activate only as a sorcery."
            .to_string(),
        abilities: vec![
            // TODO: {1}{G/P}, {T}, Sacrifice a creature: search for creature with MV = sacrificed MV + 1
            //   (needs dynamic MV filter on SearchLibrary; Phyrexian mana cost gap; sacrifice-as-cost with reference)
        ],
        completeness: Completeness::inert(
            "Blocked on a dynamic mana-value filter for SearchLibrary: MV must equal 1 + the \
             sacrificed creature's MV, and TargetFilter only has static max_cmc/min_cmc. \
             Phyrexian mana and Cost::Sacrifice are available (already used in this def's \
             mana_cost).",
        ),
        ..Default::default()
    }
}

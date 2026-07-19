// Birthing Pod — {3}{G/P} Artifact
// {1}{G/P}, {T}, Sacrifice a creature: Search your library for a creature card with mana value equal to
//   1 plus the sacrificed creature's mana value, put that card onto the battlefield, then shuffle.
//   Activate only as a sorcery.
//
// PB-OS8: TargetFilter.min_cmc_amount (runtime LOWER-BOUND cap, mirror of the existing
// max_cmc_amount) now ships, so the "mana value EQUAL TO 1 + the sacrificed creature's MV"
// filter IS expressible (max_cmc_amount == min_cmc_amount == Sum(Fixed(1),
// ManaValueOfSacrificedCreature)) — the PB-EF10-era blocker above is CLOSED. A SECOND,
// independent blocker remains: this activated ability's cost is {1}{G/P} (a Phyrexian pip),
// and Phyrexian mana is NOT handled in the activated-ability payment path (rules/abilities.rs
// has zero Phyrexian references; the "{G/P} paid with 2 life" alternative lives only in
// rules/casting.rs, for spell casting). Authoring the cost as plain {1}{G} would ship wrong
// game state (silently removes the 2-life payment option). Still blocked; recorded as
// OOS-OS8-1 (Phyrexian mana in activated-ability costs).
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
            // TODO(OOS-OS8-1): {1}{G/P}, {T}, Sacrifice a creature: search for a creature with
            //   MV = sacrificed MV + 1 (now expressible via SearchLibrary's paired
            //   max_cmc_amount/min_cmc_amount, PB-OS8). Blocked on Phyrexian mana in an
            //   ACTIVATED ability's cost — unsupported in rules/abilities.rs's payment path.
        ],
        completeness: Completeness::inert(
            "Two blockers were tracked here; ONE is now closed. CLOSED (PB-OS8): the dynamic \
             mana-value filter — SearchLibrary now honors a paired max_cmc_amount/min_cmc_amount \
             (both = Sum(Fixed(1), ManaValueOfSacrificedCreature)) to express 'MV equal to 1 plus \
             the sacrificed creature's MV'. STILL BLOCKED (OOS-OS8-1): this ability's activation \
             cost is {1}{G/P} (Phyrexian mana), and Phyrexian mana is not handled in the \
             activated-ability payment path (rules/abilities.rs has zero Phyrexian references; \
             the 2-life alternative lives only in rules/casting.rs for spell casting). Authoring \
             the cost as plain {1}{G} would ship wrong game state.",
        ),
        ..Default::default()
    }
}

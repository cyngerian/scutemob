// Birthing Ritual — {1}{G}, Enchantment
// "At the beginning of your end step, if you control a creature, look at the top
//  seven cards of your library. Then you may sacrifice a creature. If you do, you
//  may put a creature card with mana value X or less from among those cards onto
//  the battlefield, where X is 1 plus the sacrificed creature's mana value. Put
//  the rest on the bottom of your library in a random order."
//
// PB-EF10 chain-verify: the trigger (AtBeginningOfYourEndStep), intervening-if
// (YouControlPermanent(creature)), optional sacrifice (MayPayThenEffect +
// Cost::Sacrifice), and the runtime mana-value cap (Sum(Fixed(1),
// ManaValueOfSacrificedCreature) via TargetFilter.max_cmc_amount) are ALL now
// expressible after this PB. The remaining, and ONLY, blocker is the DIG itself:
// "look at the top seven, you may put ONE creature with MV <= X *from among
// those seven* onto the battlefield, put the rest on the bottom in a RANDOM
// order." No existing Effect expresses "scope candidates to a looked-at top-N
// subset of the library (not the whole library), place at most one matching a
// runtime cap, then send the remainder to the bottom in randomized order."
// `SearchLibrary` searches the WHOLE library (not a top-7 subset) and has no
// bottom-randomize destination — using it here would be legal-but-wrong (ignores
// the top-7 restriction and the bottom-random remainder), so it is NOT used.
// Filed as OOS-EF10-1 (memory/card-authoring/w-miss-engine-findings-2026-07-17.md,
// EF-W-MISS-7 section): a new `Effect::LookAtTopThenPlace` primitive is needed.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("birthing-ritual"),
        name: "Birthing Ritual".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your end step, if you control a creature, look at the \
                      top seven cards of your library. Then you may sacrifice a creature. If you \
                      do, you may put a creature card with mana value X or less from among those \
                      cards onto the battlefield, where X is 1 plus the sacrificed creature's \
                      mana value. Put the rest on the bottom of your library in a random order."
            .to_string(),
        abilities: vec![
            // TODO(OOS-EF10-1): the end-step trigger, "if you control a creature"
            // intervening-if, optional sacrifice, and runtime MV cap (1 + sacrificed
            // creature's mana value) are all expressible post-PB-EF10. Blocked ONLY on
            // the "look at top seven -> place at most one matching card from that
            // subset -> rest to bottom in random order" dig, which has no primitive.
            // Do not approximate with a whole-library SearchLibrary (legal-but-wrong:
            // ignores the top-7 scoping and the bottom-random remainder).
        ],
        completeness: Completeness::inert(
            "Blocked on OOS-EF10-1: 'look at the top seven, put at most one matching card from \
             that subset onto the battlefield (runtime MV cap = 1 + sacrificed creature's mana \
             value), put the rest on the bottom in a random order' has no Effect primitive. \
             SearchLibrary searches the whole library (not a top-7 subset) and has no \
             bottom-randomize destination -- using it would ship wrong game state. The trigger, \
             intervening-if, optional sacrifice, and runtime MV cap are ALL otherwise expressible \
             after PB-EF10 (TargetFilter.max_cmc_amount / ManaValueOfSacrificedCreature).",
        ),
        ..Default::default()
    }
}

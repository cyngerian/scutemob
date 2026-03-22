// Force of Despair — {1}{B}{B}, Instant
// If it's not your turn, you may exile a black card from your hand rather than pay this spell's mana cost.
// Destroy all creatures that entered this turn.
// TODO: DSL gap — alternative cost (exile a black card from hand) requires AltCostKind variant
// for "exile colored card from hand if not your turn"; not currently in DSL.
// TODO: DSL gap — "destroy all creatures that entered this turn" requires a TargetFilter that
// filters by entered-this-turn; DestroyAll does not support this filter predicate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("force-of-despair"),
        name: "Force of Despair".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If it's not your turn, you may exile a black card from your hand rather than pay this spell's mana cost.\nDestroy all creatures that entered this turn.".to_string(),
        abilities: vec![
            // TODO: Alternative cost — exile a black card from hand (only on opponent's turn).
            // DSL gap: no AltCostKind for "exile a colored card from hand".

            // TODO: "Destroy all creatures that entered this turn."
            // DSL gap: DestroyAll TargetFilter has no entered-this-turn predicate.
        ],
        ..Default::default()
    }
}

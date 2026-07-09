// Force of Despair — {1}{B}{B}, Instant
// If it's not your turn, you may exile a black card from your hand rather than pay this spell's mana cost.
// Destroy all creatures that entered this turn.
//
// Pitch alt cost primitive shipped in PB-AC5 (Cost::ExileFromHand + AltCostKind::Pitch).
// Card still BLOCKED: "destroy all creatures that entered this turn" requires a
// TargetFilter/DestroyAll predicate for entered-this-turn, which does not exist in the
// DSL (a separate primitive from PB-AC5's pitch cost). Do not author until that
// entered-this-turn filter predicate exists.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("force-of-despair"),
        name: "Force of Despair".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If it's not your turn, you may exile a black card from your hand rather than pay this spell's mana cost.\nDestroy all creatures that entered this turn.".to_string(),
        abilities: vec![
            // ENGINE-BLOCKED: "Destroy all creatures that entered this turn."
            // DSL gap: DestroyAll TargetFilter has no entered-this-turn predicate. The
            // pitch alt cost (exile a black card from hand, opponent's-turn-only) is
            // implemented and ready to attach once the destroy-filter gap closes, but is
            // intentionally NOT added here alone — per W6 policy, no partial authoring.
        ],
        ..Default::default()
    }
}

// Benefactor's Draught — {1}{G}, Instant
// Untap all creatures. Until end of turn, whenever a creature an opponent controls
// blocks, draw a card.
// Draw a card.
//
// ENGINE-BLOCKED: this is a SINGLE spell ability whose resolution is a sequence of three
// parts — "Untap all creatures" (now expressible via `Effect::UntapAll`), a delayed "until
// end of turn, whenever a creature an opponent controls blocks, draw a card" triggered
// ability (no delayed-trigger-creation primitive in the DSL), and "Draw a card." Per W5
// policy, a single ability's effect list cannot be partially authored: authoring only the
// untap and the unconditional draw while silently dropping the delayed block-trigger would
// produce wrong game state (the caster would never get the "draw on opponent block" upside
// the real card grants for the rest of the turn). Left fully blocked until a
// delayed-triggered-ability-creation primitive ships.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("benefactors-draught"),
        name: "Benefactor's Draught".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Untap all creatures. Until end of turn, whenever a creature an opponent \
                      controls blocks, draw a card.\nDraw a card."
            .to_string(),
        abilities: vec![],
        completeness: Completeness::inert(
            "this is a SINGLE spell ability whose resolution is a sequence of three parts — \
             'Untap all creatures' (now expressible...",
        ),
        ..Default::default()
    }
}

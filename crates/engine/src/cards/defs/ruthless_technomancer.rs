// Ruthless Technomancer — {3}{B}, Creature — Human Wizard 2/4
// When this creature enters, you may sacrifice another creature you control. If you do,
// create a number of Treasure tokens equal to that creature's power.
// {2}{B}, Sacrifice X artifacts: Return target creature card with power X or less from
// your graveyard to the battlefield. X can't be 0.
//
// ENGINE-BLOCKED (ETB): "you may sacrifice another creature you control. If you do,
// create a number of Treasure tokens equal to that creature's power." PB-AC2's
// Effect::MayPayThenEffect (CR 118.12) can express the "may sacrifice -> then"
// shape, but (1) Cost::Sacrifice(filter) has no "another" / exclude-self semantics
// (can_pay_optional_cost/pay_optional_cost thread only a PlayerId, not a source
// ObjectId, so the source itself -- a creature -- could be offered as the
// sacrifice), and (2) there is no EffectAmount/context plumbing that captures the
// sacrificed permanent's power for a dynamically-sized CreateToken count. Both
// gaps are genuine (verified against card_definition.rs / effects/mod.rs).
//
// ENGINE-BLOCKED (activated ability): "{2}{B}, Sacrifice X artifacts: Return target
// creature card with power X or less from your graveyard to the battlefield. X
// can't be 0." No Cost variant supports a player-chosen variable-X sacrifice count,
// and TargetFilter has no "power <= X" (dynamic, tied to the chosen X) graveyard
// target filter. Out of PB-AC2 scope.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ruthless-technomancer"),
        name: "Ruthless Technomancer".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "When this creature enters, you may sacrifice another creature you control. If you do, create a number of Treasure tokens equal to that creature's power.\n{2}{B}, Sacrifice X artifacts: Return target creature card with power X or less from your graveyard to the battlefield. X can't be 0.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // ENGINE-BLOCKED: see module comment -- ETB optional-sacrifice-for-Treasure
            // and the activated variable-X ability are both blocked on real DSL gaps.
        ],
        completeness: Completeness::partial("(ETB): 'you may sacrifice another creature you control. If you do, create a number of Treasure tokens equal to that..."),
        ..Default::default()
    }
}

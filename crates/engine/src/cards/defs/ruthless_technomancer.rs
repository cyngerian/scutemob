// Ruthless Technomancer — {3}{B}, Creature — Human Wizard 2/4
// When this creature enters, you may sacrifice another creature you control. If you do,
// create a number of Treasure tokens equal to that creature's power.
// {2}{B}, Sacrifice X artifacts: Return target creature card with power X or less from
// your graveyard to the battlefield. X can't be 0.
//
// TODO: ETB trigger "you may sacrifice another creature you control. If you do, create
// Treasure tokens equal to that creature's power." — optional sacrifice with conditional
// effect and EffectAmount based on sacrificed creature's power not in DSL.
//
// TODO: "{2}{B}, Sacrifice X artifacts: Return target creature card with power X or less
// from your graveyard to the battlefield." — Cost::Sacrifice with variable X count, and
// TargetFilter::CreatureInGraveyardWithPowerXOrLess not in DSL.
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
            // TODO: ETB optional sacrifice + create Treasure equal to sacrificed creature's power
            // TODO: Activated {2}{B} + Sacrifice X artifacts, graveyard recursion with power filter
        ],
        ..Default::default()
    }
}

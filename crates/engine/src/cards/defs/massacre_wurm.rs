// Massacre Wurm
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("massacre-wurm"),
        name: "Massacre Wurm".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 3, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Wurm"]),
        oracle_text: "When this creature enters, creatures your opponents control get -2/-2 until end of turn.
Whenever a creature an opponent controls dies, that player loses 2 life.".to_string(),
        power: Some(6),
        toughness: Some(5),
        abilities: vec![
            // TODO: DSL gap — ETB: "creatures your opponents control get -2/-2 until EOT."
            // EffectFilter::CreaturesOpponentsControl does not exist.
            // TODO: DSL gap — "Whenever a creature an opponent controls dies" trigger.
            // WheneverCreatureDies has no controller filter.
        ],
        ..Default::default()
    }
}

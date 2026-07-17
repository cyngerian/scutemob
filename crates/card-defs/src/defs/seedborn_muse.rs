// Seedborn Muse — {3}{G}{G} Creature — Spirit 2/4
// Untap all permanents you control during each other player's untap step.
//
// DSL gap: "untap all permanents you control during each other player's untap step" is a
//   static/replacement effect on the untap step (not a triggered ability firing on this permanent's
//   controller's untap). No UntapAllYouControl in DSL for other players' untap steps.
// W5 policy: cannot faithfully express this — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("seedborn-muse"),
        name: "Seedborn Muse".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: creature_types(&["Spirit"]),
        oracle_text: "Untap all permanents you control during each other player's untap step.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // TODO: untap all permanents you control during each other player's untap step
            //   (no TriggerCondition for "during opponent's untap step" + Effect::UntapAllYouControl)
        ],
        completeness: Completeness::inert("'untap all permanents you control during each other player's untap step' is the card's only ability and is omitted, so this def registers no behaviour at all (CR: Inert, not KnownWrong — nothing is implemented-but-deviating). Blocked solely on the hook — no TriggerCondition/TriggerEvent for another player's untap step (Effect::UntapAll{filter} already exists for the effect half)."),
        ..Default::default()
    }
}

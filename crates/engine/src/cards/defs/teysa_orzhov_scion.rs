// Teysa, Orzhov Scion — {1}{W}{B}, Legendary Creature — Human Advisor 2/3
// Sacrifice three white creatures: Exile target creature.
// Whenever another black creature you control dies, create a 1/1 white Spirit creature token
// with flying.
// TODO: Sacrifice ability — no DSL Cost variant for "sacrifice N permanents of a given type"
//   (only Cost::Sacrifice sacrifices one). "Sacrifice three white creatures" as an activated
//   cost is not expressible.
// TODO: Death trigger — "whenever another black creature you control dies" requires filtering
//   the dying creature by color (black) and controller (you). WheneverCreatureDies is overbroad
//   (triggers on any creature death). No color filter on the trigger condition exists in DSL.
//   Per W5 policy, leaving abilities empty rather than producing wrong game state.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("teysa-orzhov-scion"),
        name: "Teysa, Orzhov Scion".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Advisor"]),
        oracle_text: "Sacrifice three white creatures: Exile target creature.\nWhenever another black creature you control dies, create a 1/1 white Spirit creature token with flying.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![],
        ..Default::default()
    }
}

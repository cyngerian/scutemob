// Song of Freyalise — {1}{G} Enchantment — Saga
// (As this Saga enters and after your draw step, add a lore counter. Sacrifice after III.)
// I, II — Until your next turn, creatures you control gain "{T}: Add one mana of any color."
// III — Put a +1/+1 counter on each creature you control. Those creatures gain vigilance,
//        trample, and indestructible until end of turn.
//
// DSL gap: Saga chapter trigger mechanics (lore counters, chapter thresholds) not in DSL.
//   Chapter I/II grants an activated ability to all creatures (GrantActivatedAbility gap).
//   Chapter III puts counters on all creatures + grants keywords until end of turn (partially
//   expressible but needs Saga framework).
// W5 policy: cannot faithfully express Saga chapters — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("song-of-freyalise"),
        name: "Song of Freyalise".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Saga"]),
        oracle_text: "(As this Saga enters and after your draw step, add a lore counter. Sacrifice after III.)\nI, II — Until your next turn, creatures you control gain \"{T}: Add one mana of any color.\"\nIII — Put a +1/+1 counter on each creature you control. Those creatures gain vigilance, trample, and indestructible until end of turn.".to_string(),
        abilities: vec![
            // TODO: Saga chapter I/II — until your next turn creatures you control have mana tap ability
            //   (Saga framework not in DSL; also needs GrantActivatedAbility)
            // TODO: Saga chapter III — +1/+1 counters on all creatures + vigilance/trample/indestructible until EOT
            //   (Saga framework not in DSL)
        ],
        ..Default::default()
    }
}

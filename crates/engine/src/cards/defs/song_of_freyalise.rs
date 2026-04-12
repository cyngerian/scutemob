// Song of Freyalise — {1}{G} Enchantment — Saga
// (As this Saga enters and after your draw step, add a lore counter. Sacrifice after III.)
// I, II — Until your next turn, creatures you control gain "{T}: Add one mana of any color."
// III — Put a +1/+1 counter on each creature you control. Those creatures gain vigilance,
//        trample, and indestructible until end of turn.
//
// Blocked on Saga framework: lore counters, chapter ability trigger thresholds
// (CR 714) are not yet in the DSL. The grant primitives needed for chapter
// effects (LayerModification::AddManaAbility + UntilYourNextTurn duration) all
// exist post-PB-S — only the chapter machinery itself is missing.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("song-of-freyalise"),
        name: "Song of Freyalise".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Saga"]),
        oracle_text: "(As this Saga enters and after your draw step, add a lore counter. Sacrifice after III.)\nI, II — Until your next turn, creatures you control gain \"{T}: Add one mana of any color.\"\nIII — Put a +1/+1 counter on each creature you control. Those creatures gain vigilance, trample, and indestructible until end of turn.".to_string(),
        abilities: vec![
            // TODO: Saga chapter I/II — grant `{T}: Add one mana of any color` to creatures you
            //   control until your next turn. Effect primitives (AddManaAbility, UntilYourNextTurn)
            //   exist; blocked on Saga chapter framework (CR 714 lore counters / chapter triggers).
            // TODO: Saga chapter III — +1/+1 counters on all creatures you control plus vigilance,
            //   trample, indestructible until EOT. Blocked on Saga chapter framework.
        ],
        ..Default::default()
    }
}

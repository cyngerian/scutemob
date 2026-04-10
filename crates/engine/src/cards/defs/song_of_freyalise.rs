// Song of Freyalise — {1}{G} Enchantment — Saga
// (As this Saga enters and after your draw step, add a lore counter. Sacrifice after III.)
// I, II — Until your next turn, creatures you control gain "{T}: Add one mana of any color."
// III — Put a +1/+1 counter on each creature you control. Those creatures gain vigilance,
//        trample, and indestructible until end of turn.
//
// Partially unblocked by PB-S: chapter I/II can now use
//   LayerModification::AddManaAbility + EffectFilter::CreaturesYouControl + EffectDuration::UntilYourNextTurn
// Remaining blocker: Saga chapter trigger framework (lore counters, chapter thresholds)
// is not yet in DSL. When the Saga PB lands, chapters I/II can use AddManaAbility directly.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("song-of-freyalise"),
        name: "Song of Freyalise".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Saga"]),
        oracle_text: "(As this Saga enters and after your draw step, add a lore counter. Sacrifice after III.)\nI, II — Until your next turn, creatures you control gain \"{T}: Add one mana of any color.\"\nIII — Put a +1/+1 counter on each creature you control. Those creatures gain vigilance, trample, and indestructible until end of turn.".to_string(),
        abilities: vec![
            // TODO: Saga chapter I/II — grant via PB-S LayerModification::AddManaAbility
            //   with EffectFilter::CreaturesYouControl + EffectDuration::UntilYourNextTurn.
            //   Unblocked by PB-S; still blocked on Saga framework (lore counters / chapter
            //   trigger thresholds not in DSL).
            // TODO: Saga chapter III — +1/+1 counters on all creatures + keywords until EOT.
            //   Blocked on Saga framework.
        ],
        ..Default::default()
    }
}

// Season of Gathering — {4}{G}{G} Sorcery
// Choose up to five {P} worth of modes. You may choose the same mode more than once.
// {P} — Put a +1/+1 counter on a creature you control. It gains vigilance and trample until end of turn.
// {P}{P} — Choose artifact or enchantment. Destroy all permanents of the chosen type.
// {P}{P}{P} — Draw cards equal to the greatest power among creatures you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("season-of-gathering"),
        name: "Season of Gathering".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose up to five {P} worth of modes. You may choose the same mode more than once.\n\
            {P} — Put a +1/+1 counter on a creature you control. It gains vigilance and trample until end of turn.\n\
            {P}{P} — Choose artifact or enchantment. Destroy all permanents of the chosen type.\n\
            {P}{P}{P} — Draw cards equal to the greatest power among creatures you control."
            .to_string(),
        // TODO: Season of Gathering uses Phyrexian mana ({P}) as a mode budget (up to 5 {P}
        // total, repeatable modes). This requires:
        //   1. A mode-budget system where each mode costs a different amount of the budget.
        //   2. allow_duplicate_modes = true for modes 0 and 1.
        //   3. Mode 0: AddCounter + temporary keyword grants (Vigilance + Trample) on target creature.
        //   4. Mode 1: Player choice of "artifact or enchantment" then DestroyAll of that type.
        //   5. Mode 2: DrawCards where count = greatest power among creatures you control (dynamic count).
        // None of these combinations are currently expressible together in the DSL.
        abilities: vec![],
        ..Default::default()
    }
}

// Battle Cry Goblin — {1}{R}, Creature — Goblin 2/2
// {1}{R}: Goblins you control get +1/+0 and gain haste until end of turn.
// Pack tactics — Whenever this creature attacks, if you attacked with creatures with total
// power 6 or greater this combat, create a 1/1 red Goblin creature token that's tapped
// and attacking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("battle-cry-goblin"),
        name: "Battle Cry Goblin".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin"]),
        oracle_text: "{1}{R}: Goblins you control get +1/+0 and gain haste until end of turn.\nPack tactics \u{2014} Whenever this creature attacks, if you attacked with creatures with total power 6 or greater this combat, create a 1/1 red Goblin creature token that's tapped and attacking.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "{1}{R}: Goblins you control get +1/+0 and gain haste until end of turn."
            // DSL gap: ApplyContinuousEffect targeting all Goblins you control (+1/+0 only)
            // requires EffectFilter::CreaturesYouControlWithSubtype + LayerModification::ModifyPower
            // (not ModifyBoth). ModifyPower alone is not currently available.
            //
            // TODO: Pack tactics — "Whenever this creature attacks, if you attacked with
            // creatures with total power 6 or greater this combat, create a 1/1 red Goblin
            // creature token that's tapped and attacking."
            // DSL gap: Condition::AttackedWithTotalPowerAtLeast(6) does not exist.
            // Cannot implement trigger without the intervening-if — firing unconditionally
            // would produce wrong game state (W5 policy).
        ],
        ..Default::default()
    }
}

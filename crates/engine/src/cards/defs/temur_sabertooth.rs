// Temur Sabertooth — {2}{G}{G}, Creature — Cat 4/3
// {1}{G}: You may return another creature you control to its owner's hand. If you do, this
// creature gains indestructible until end of turn.
//
// TODO: DSL gap — conditional "if you do" branching on returning another creature is not
// expressible. The bounce + conditional self-buff pattern requires a MayDo construct with
// a follow-up effect tied to the optional bounce. Omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("temur-sabertooth"),
        name: "Temur Sabertooth".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: creature_types(&["Cat"]),
        oracle_text: "{1}{G}: You may return another creature you control to its owner's hand. If you do, this creature gains indestructible until end of turn.".to_string(),
        power: Some(4),
        toughness: Some(3),
        abilities: vec![
            // TODO: {1}{G} activated — bounce another creature + conditional indestructible until EOT
        ],
        ..Default::default()
    }
}

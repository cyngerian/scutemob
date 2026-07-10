// Temur Sabertooth — {2}{G}{G}, Creature — Cat 4/3
// {1}{G}: You may return another creature you control to its owner's hand. If you do, this
// creature gains indestructible until end of turn.
//
// ENGINE-BLOCKED — out of PB-AC2 scope: PB-AC2's `Effect::MayPayThenEffect` only wraps
// `Cost` variants (Mana/PayLife/DiscardCard/Sacrifice/Sequence). The optional action
// here is "return another creature you control to its owner's hand" — a bounce, not a
// `Cost`. No `Cost::ReturnPermanentToHand` (or similar) variant exists, so this cannot
// be expressed as a beneficial-pay wrapper. Genuinely missing; not a MayPayThenEffect gap.
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
            // ENGINE-BLOCKED: see module comment — bounce-as-cost has no Cost variant.
        ],
        completeness: Completeness::partial("out of PB-AC2 scope: PB-AC2's `Effect::MayPayThenEffect` only wraps `Cost` variants..."),
        ..Default::default()
    }
}

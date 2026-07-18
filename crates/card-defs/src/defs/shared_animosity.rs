// Shared Animosity — {2}{R}, Enchantment
// Whenever a creature you control attacks, it gets +1/+0 until end of turn for each
// other attacking creature that shares a creature type with it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shared-animosity"),
        name: "Shared Animosity".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control attacks, it gets +1/+0 until end of turn \
                      for each other attacking creature that shares a creature type with it."
            .to_string(),
        abilities: vec![
            // CR 508.1m / CR 603.2: "Whenever a creature you control attacks, it gets +1/+0
            // until end of turn for each other attacking creature that shares a creature type."
            // WheneverCreatureYouControlAttacks exists (PB-N) — the trigger condition is now
            // expressible.
            // TODO: DSL gap — EffectAmount has no variant for "count of other attacking
            // creatures that share a creature type with the triggering creature". This requires
            // a dynamic per-trigger count keyed on the triggering creature's subtypes vs. all
            // other attackers' subtypes. No EffectAmount::CountOtherAttackersWithSharedSubtype
            // or equivalent exists. The buff target (the triggering creature) also requires
            // EffectFilter::TriggeringCreature in ContinuousEffectDef. Both gaps must be filled
            // before this ability can be expressed without producing wrong game state.
        ],
        completeness: Completeness::inert(
            "DSL gap — EffectAmount has no variant for 'count of other attacking creatures that \
             share a creature type with the...",
        ),
        ..Default::default()
    }
}

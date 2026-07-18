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
            // expressible, and PB-EF4 closed the buff-target gap: EffectFilter::TriggeringCreature
            // now exists, so the +1/+0 grant CAN be aimed at the attacking creature.
            // TODO: DSL gap (OOS-EF4-1) — EffectAmount still has no variant for "count of other
            // attacking creatures that share a creature type with the triggering creature". This
            // requires a dynamic per-trigger count keyed on the triggering creature's
            // layer-resolved subtypes vs. every other attacker's subtypes. No
            // EffectAmount::CountOtherAttackersWithSharedSubtype or equivalent exists. Authoring
            // with a fixed/wrong amount would ship incorrect game state (PB-EF4 plan, "do NOT
            // substitute a gated Effect::Choose/fixed count").
        ],
        completeness: Completeness::inert(
            "OOS-EF4-1: DSL gap — EffectAmount has no variant for 'count of other attacking \
             creatures that share a creature type with the triggering creature' (a dynamic \
             per-trigger count). EffectFilter::TriggeringCreature (PB-EF4) closed the buff-target \
             half; only the count-amount half remains blocked.",
        ),
        ..Default::default()
    }
}

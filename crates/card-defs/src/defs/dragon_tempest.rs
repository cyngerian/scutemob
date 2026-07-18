// Dragon Tempest — {1}{R}, Enchantment
// Whenever a creature you control with flying enters, it gains haste until end of turn.
// Whenever a Dragon you control enters, it deals X damage to any target, where X is the
// number of Dragons you control.
//
// Both triggered abilities target "it" = the ENTERING creature, never Dragon Tempest itself
// (Dragon Tempest is an Enchantment, not a creature, so it can never be the "it"). Neither
// clause is faithfully expressible in the current DSL:
// 1. "it gains haste until end of turn" needs Effect::ApplyContinuousEffect to grant a keyword
//    to the specific triggering creature — ContinuousEffectDef.filter is EffectFilter, which
//    has no TriggeringCreature variant (confirmed: ogre_battledriver.rs / shared_animosity.rs
//    hit the identical gap and are marked partial for the same reason).
// 2. "it deals X damage" needs the damage to be sourced from the entering Dragon, but
//    Effect::DealDamage has no source-override field — it always sources from ctx.source
//    (Dragon Tempest), which here is NEVER the "it" the oracle means. Unlike Scourge of
//    Valkas (where a "this creature enters" half lets ctx.source coincide with "it"), Dragon
//    Tempest has no self case at all — every firing of this trigger is by definition "another"
//    creature (Dragon Tempest is not itself a Dragon). Implementing it would misattribute the
//    damage source on 100% of triggers, not just an edge case.
// Both abilities are omitted to avoid shipping wrong game state (W5).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragon-tempest"),
        name: "Dragon Tempest".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control with flying enters, it gains haste until \
                      end of turn.\nWhenever a Dragon you control enters, it deals X damage to \
                      any target, where X is the number of Dragons you control."
            .to_string(),
        abilities: vec![],
        completeness: Completeness::inert(
            "Both clauses target 'it' = the entering creature, never Dragon Tempest itself. (1) \
             'it gains haste' needs EffectFilter::TriggeringCreature on \
             ContinuousEffectDef.filter, which doesn't exist (same gap as ogre_battledriver.rs / \
             shared_animosity.rs). (2) 'it deals X damage' needs the entering Dragon to be the \
             damage source, but Effect::DealDamage always sources from ctx.source (Dragon \
             Tempest) — and unlike Scourge of Valkas, Dragon Tempest has no self-ETB case to fall \
             back on (it is not itself a Dragon), so every trigger firing would misattribute the \
             source. Both left unauthored per W5.",
        ),
        ..Default::default()
    }
}

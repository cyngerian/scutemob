// Marisi, Breaker of the Coil — {1}{R}{G}{W}, Legendary Creature — Cat Warrior 5/4
// Your opponents can't cast spells during combat.
// Whenever a creature you control deals combat damage to a player, goad each creature
// that player controls.
//
// Clause 2 (goad) is authored below (CR 701.15a, CR 510.3a) — see the Triggered ability.
// Clause 1 ("Your opponents can't cast spells during combat") stays UNAUTHORED: it needs a
// phase-scoped GameRestriction and all 11 `GameRestriction` variants (stubs.rs:558-612) are
// turn- or count-scoped only, none phase-scoped. Genuinely blocked, not a stale claim.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("marisi-breaker-of-the-coil"),
        name: "Marisi, Breaker of the Coil".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            green: 1,
            white: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Cat", "Warrior"],
        ),
        oracle_text: "Your opponents can't cast spells during combat.\nWhenever a creature you \
                      control deals combat damage to a player, goad each creature that player \
                      controls."
            .to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            // CR 510.3a / CR 701.15a: "Whenever a creature you control deals combat damage to a
            // player, goad each creature that player controls." DamagedPlayer scopes both the
            // trigger's per-creature firing and the goad target set to the specific player dealt
            // damage (multiplayer-exact — mirrors throat_slitter.rs / balefire_dragon.rs).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition:
                    TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer {
                        filter: None,
                    },
                effect: Effect::Goad {
                    target: EffectTarget::AllPermanentsMatching(Box::new(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::DamagedPlayer,
                        ..Default::default()
                    })),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::partial(
            "Clause 2 (goad, CR 701.15a) is authored. Blocked on clause 1 only: 'Your opponents \
             can't cast spells during combat' needs a phase-scoped GameRestriction, and all 11 \
             GameRestriction variants (stubs.rs:558-612) are turn- or count-scoped, none \
             phase-scoped.",
        ),
        ..Default::default()
    }
}

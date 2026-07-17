// Master Biomancer — {2}{G}{U}, Creature — Elf Wizard 2/4
// Each other creature you control enters with a number of additional +1/+1 counters on it
//   equal to this creature's power and as a Mutant in addition to its other types.
//
// PB-EWC (2026-05-14): authored the counter half via
// `ReplacementModification::EntersWithCounters { counter: PlusOnePlusOne,
//   count: EffectAmount::PowerOf(EffectTarget::Source) }`. The replacement's
// source is Master Biomancer itself (registered with `source: Some(new_id)` at
// register_permanent_replacement_abilities), so the resolver reads
// MB's live, layer-resolved power (CR 614.12: "as it would exist on the
// battlefield"). Ruling 2013-01-24: "use Master Biomancer's power as that
// creature is entering" — exactly the live resolver semantic.
//
// PB-EAT (2026-05-15): authors the type-grant half via
// `ReplacementModification::EntersAsAdditionalType { subtype: SubType("Mutant") }`.
// CR 614.1c entry modification: the subtype is pushed into the entering permanent's
// `characteristics.subtypes` BEFORE `PermanentEnteredBattlefield` is emitted, so
// ETB triggers and SBAs observe the augmented type set on the very turn it enters.
// This is NOT a Layer 4 continuous type-adding effect (which would only apply to
// permanents already on the battlefield and would not alter the entering object's
// own characteristics at ETB time).
//
// Why "each OTHER creature" is correct without an explicit exclude_self for BOTH
// replacements: `register_permanent_replacement_abilities` runs AFTER
// `apply_etb_replacements` for the same ETB. MB's replacements are therefore
// not registered in time to fire on its own ETB. Any subsequent creature ETB
// matches CreatureControlledBy(controller) — and is, by construction, a
// different ObjectId from MB.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("master-biomancer"),
        name: "Master Biomancer".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Elf", "Wizard"]),
        oracle_text: "Each other creature you control enters with a number of additional +1/+1 \
                      counters on it equal to this creature's power and as a Mutant in addition \
                      to its other types."
            .to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // CR 614.1c + 614.12 — replacement effect: "Each other creature you
            // control enters with a number of additional +1/+1 counters on it
            // equal to this creature's power."
            //
            // `CreatureControlledBy(PlayerId(0))` is the placeholder bound to
            // the actual controller by `bind_object_filter` at registration.
            // Layer-resolved type check (CR 613.1d) is performed in
            // `object_matches_filter`.
            //
            // `EffectAmount::PowerOf(EffectTarget::Source)` reads the source's
            // layer-resolved power via `calculate_characteristics` — live, not
            // LKI — which captures anthems and counters on Biomancer.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::CreatureControlledBy(PlayerId(0)),
                },
                modification: ReplacementModification::EntersWithCounters {
                    counter: CounterType::PlusOnePlusOne,
                    count: Box::new(EffectAmount::PowerOf(EffectTarget::Source)),
                },
                is_self: false,
                unless_condition: None,
            },
            // CR 614.1c + CR 205.3 — replacement effect: "...and as a Mutant in
            // addition to its other types."
            //
            // Same `CreatureControlledBy(PlayerId(0))` placeholder; same
            // bind_object_filter rebinding to the actual controller. Both
            // replacements fire on the same ETB event (the counter and type
            // modifications are independent — CR 614.5 forbids double-applying
            // a single replacement, not two distinct replacements from the
            // same source).
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::CreatureControlledBy(PlayerId(0)),
                },
                modification: ReplacementModification::EntersAsAdditionalType {
                    subtype: SubType("Mutant".to_string()),
                },
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}

// Master Biomancer — {2}{G}{U}, Creature — Elf Wizard 2/4
// Each other creature you control enters with a number of additional +1/+1 counters on it
//   equal to this creature's power and as a Mutant in addition to its other types.
//
// PB-EWC (2026-05-14): authors the counter half via
// `ReplacementModification::EntersWithCounters { counter: PlusOnePlusOne,
//   count: EffectAmount::PowerOf(EffectTarget::Source) }`. The replacement's
// source is Master Biomancer itself (registered with `source: Some(new_id)` at
// register_permanent_replacement_abilities), so the resolver reads
// MB's live, layer-resolved power (CR 614.12: "as it would exist on the
// battlefield"). Ruling 2013-01-24: "use Master Biomancer's power as that
// creature is entering" — exactly the live resolver semantic.
//
// Why "each OTHER creature" is correct without an explicit exclude_self:
//   `register_permanent_replacement_abilities` runs AFTER
//   `apply_etb_replacements` for the same ETB. MB's replacement is therefore
//   not registered in time to fire on its own ETB. Any subsequent creature
//   ETB matches CreatureControlledBy(controller) — and is, by construction,
//   a different ObjectId from MB.
//
// TODO (OOS-EWC-1): the type-grant half ("as a Mutant in addition to its other
// types") is a separate replacement primitive (EntersAsAdditionalType) that
// PB-EWC does not ship. Filed in memory/primitives/pb-retriage-CC.md.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("master-biomancer"),
        name: "Master Biomancer".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Wizard"]),
        oracle_text: "Each other creature you control enters with a number of additional +1/+1 counters on it equal to this creature's power and as a Mutant in addition to its other types.".to_string(),
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
            // TODO (OOS-EWC-1): "as a Mutant in addition to its other types"
            // requires a new ReplacementModification primitive
            // (EntersAsAdditionalType). Filed in memory/primitives/pb-retriage-CC.md.
        ],
        ..Default::default()
    }
}

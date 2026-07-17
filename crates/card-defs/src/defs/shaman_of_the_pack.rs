// Shaman of the Pack — {1}{B}{G}, Creature — Elf Shaman 3/2
// When this creature enters, target opponent loses life equal to the number of Elves
// you control.
//
// The LoseLife amount IS expressible: EffectAmount::PermanentCount { filter: { has_subtype:
// Elf, controller: You }, controller: PlayerTarget::Controller }. Still blocked on "target
// opponent": TargetRequirement has no TargetOpponent variant anywhere in the DSL (confirmed
// against raiders_wake.rs / forbidden_orchard.rs / ajani_sleeper_agent.rs, all carrying the
// same open TODO); TargetRequirement::TargetPlayer would let the controller legally target
// themselves, which is wrong game state per W5/KI-1 (an illegal target becomes legal).
// Omitted per W5 policy rather than shipped with the wrong target restriction.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shaman-of-the-pack"),
        name: "Shaman of the Pack".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 1,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Elf", "Shaman"]),
        oracle_text: "When this creature enters, target opponent loses life equal to the number \
                      of Elves you control."
            .to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // ETB "target opponent loses life = Elves you control": the AMOUNT is expressible
            // (EffectAmount::PermanentCount{ has_subtype: Elf, controller: You }), but the
            // TARGET is not — TargetRequirement has no opponent-restricted player variant, and
            // bare TargetPlayer would let the caster illegally self-target (KI-1). Omitted to
            // avoid wrong game state; blocker is the missing target requirement, not the count.
        ],
        completeness: Completeness::partial(
            "ETB target-loses-life is un-shippable: the count is expressible via \
             EffectAmount::PermanentCount{ has_subtype: Elf, controller: You }, but 'target \
             OPPONENT' has no TargetRequirement variant (only unqualified TargetPlayer, which \
             permits an illegal self-target — KI-1). Real blocker is the missing \
             opponent-restricted player target, filed as W-PB2 engine finding EF-W-PB2-2 (shared \
             with raiders_wake, forbidden_orchard, ajani_sleeper_agent).",
        ),
        ..Default::default()
    }
}

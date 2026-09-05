// Well of Lost Dreams — {4}, Artifact
// Whenever you gain life, you may pay {X}, where X is less than or equal to the amount of
//   life you gained. If you do, draw X cards.
//
// TODO: "Whenever you gain life, pay {X} up to the amount gained, draw X cards".
//   TriggerCondition::WheneverYouGainLife EXISTS and is wired (declared
//   card_definition.rs:3729, lowered in replay_harness.rs's trigger loop, exercised by
//   tests/rules/trigger_variants.rs) — an earlier draft of this note said it did not, spelling
//   it `WhenYouGainLife`, and the def's own completeness note already recorded that half as
//   false. The surviving blockers are the AMOUNT and the CAP: no EffectAmount carries the life
//   gained by the triggering event, and no Cost::PayUpToX(cap) variant exists. Omitted per W5
//   policy. (PB-DX57 / OOS-DX28-6: repaired in place rather than deleted — a stale note that
//   MISSPELLS the identifier it is wrong about is invisible to every needle-based sweep in the
//   tree, and this one survived PB-DX27's blocker sweep for exactly that reason.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("well-of-lost-dreams"),
        name: "Well of Lost Dreams".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Whenever you gain life, you may pay {X}, where X is less than or equal to \
                      the amount of life you gained. If you do, draw X cards."
            .to_string(),
        abilities: vec![
            // TODO: TriggerCondition::WheneverYouGainLife IS in the DSL (the old note said
            //   "WhenYouGainLife not in DSL" and was false in both the spelling and the claim).
            //   What is still missing is EffectAmount::LifeGainedThisEvent to cap the X cost,
            //   and a Cost::PayUpToX variant. Two DSL gaps. Omitted per W5 policy.
        ],
        completeness: Completeness::inert(
            "Blocked on (a) no EffectAmount carries the amount of life gained by the triggering \
             event, (b) no Cost::PayUpToX(cap) variant, (c) the printed 'you may pay {X}' needs a \
             VARIABLE cap. Clause (c) is NARROWED as of PB-DX57: the CR 118.12 optional-cost \
             channel itself exists since PB-DX45 (EffectChoiceQuestion::PayOptionalCost, \
             crates/card-types/src/state/stubs.rs:1030, asked by Effect::MayPayThenEffect), so \
             'you may pay has no interactive expression' is no longer the blocker — the blocker \
             is that PayOptionalCost carries a FIXED Cost, not a cap. The WheneverYouGainLife \
             trigger DOES exist and is fully wired — that half of the old note was false.",
        ),
        ..Default::default()
    }
}

// Guardian Project — {3}{G}, Enchantment
// Whenever a nontoken creature you control enters, if it doesn't have the same name as
// another creature you control or a creature card in your graveyard, draw a card.
//
// TODO: "nontoken" filter — TargetFilter lacks non_token field.
// TODO: Intervening-if "doesn't share name" — name-uniqueness condition not in DSL.
// Implementing as unconditional creature-ETB draw (overbroad approximation).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("guardian-project"),
        name: "Guardian Project".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a nontoken creature you control enters, if it doesn't have the \
                      same name as another creature you control or a creature card in your \
                      graveyard, draw a card."
            .to_string(),
        abilities: vec![
            // TODO: Should be nontoken only + name-uniqueness intervening-if.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        // PB-DX3b (OOS-DX3-1, 2026-08-01): RE-VERIFIED — and the old note's (a) half is
        // STALE, not merely re-affirmed. `TargetFilter.is_nontoken` is NOT ignored by the
        // ETB trigger path: `testing/replay_harness.rs`'s
        // `build_face_ability_vectors` forwards the def's full filter as
        // `triggering_creature_filter` for exactly this `TriggerCondition::`
        // `WheneverCreatureEntersBattlefield` shape (comment there: "PB-AC0 ... forward
        // the full carddef TargetFilter as triggering_creature_filter so has_subtype /
        // has_subtypes / is_nontoken / exclude_subtypes are honored"), and
        // `rules/abilities.rs`'s creature-ETB dispatch explicitly checks
        // `creature_filter.is_nontoken && entering_obj.is_token` before matching (a
        // runtime GameObject field matches_filter itself cannot see — exactly the same
        // pattern as `is_attacking`/`exclude_self`, not a gap). So `is_nontoken: true`
        // on this def's filter is authorable TODAY, zero engine lines.
        //
        // The (b) half remains genuinely blocked: RE-VERIFIED, no name-uniqueness
        // Condition variant exists in the current enum (checked this batch).
        //
        // DEFERRED THIS BATCH regardless: fixing only (a) does not reach `Complete` (the
        // name-uniqueness half still overdraws in real games — e.g. two same-named
        // nontoken creatures), so it stays `known_wrong`, and PB-DX3b's declared scope
        // is the four defs in its Steps 1-4. Filing the (a) half as a small follow-up
        // finding rather than fixing it silently here.
        completeness: Completeness::known_wrong(
            "draws on EVERY creature-you-control ETB. Missing (a) the nontoken restriction and \
             (b) the 'doesn't share a name with another creature you control or a creature card \
             in your graveyard' intervening-if — no name-uniqueness Condition exists. Overdraws \
             vs. the real card. (TargetFilter.is_nontoken is checked by the ETB trigger path in \
             rules/abilities.rs, NOT ignored — see the PB-DX3b re-verification note above; the \
             remaining blocker is (b) only.)",
        ),
        ..Default::default()
    }
}

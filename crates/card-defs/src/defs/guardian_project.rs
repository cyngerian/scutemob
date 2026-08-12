// Guardian Project — {3}{G}, Enchantment
// Whenever a nontoken creature you control enters, if it doesn't have the same name as
// another creature you control or a creature card in your graveyard, draw a card.
//
// The "nontoken" filter IS expressible and is authored below (TargetFilter.is_nontoken —
//   PB-DX26 / OOS-DX3b-1, 2026-08-11). The old TODO here claimed TargetFilter lacked the
//   field; it never did, and the ETB trigger path has honoured it since PB-AC0.
// TODO: Intervening-if "doesn't share name" — name-uniqueness condition not in DSL
//   (re-verified against the current Condition enum 2026-08-11).
// Implementing as a name-unconditional nontoken creature-ETB draw (overbroad approximation).
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
            // PB-DX26 (OOS-DX3b-1, 2026-08-11): the nontoken half is AUTHORED — the
            // remaining approximation is the name-uniqueness intervening-if only.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        // CR 111.1 / the printed "nontoken creature you control". RE-VERIFIED
                        // this batch, not copied forward: `testing/replay_harness.rs`'s
                        // `build_face_ability_vectors` forwards this whole filter as
                        // `triggering_creature_filter` for exactly this trigger shape, and
                        // `rules/abilities.rs`'s creature-ETB dispatch checks
                        // `creature_filter.is_nontoken && entering_obj.is_token` BEFORE
                        // matching — `is_token` is a runtime `GameObject` field that
                        // `matches_filter` itself cannot see, which is why the pre-check
                        // exists (same pattern as `is_attacking` / `exclude_self`).
                        is_nontoken: true,
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
        // PB-DX26 (OOS-DX3b-1, 2026-08-11): the (a) half PB-DX3b deferred is now APPLIED —
        // `is_nontoken: true` is set on the trigger's filter above. (b) was re-verified
        // against the CURRENT `Condition` enum this batch, not copied forward: still no
        // name-uniqueness variant, so the def stays `known_wrong`. The note below now names
        // the ONE remaining approximation; do not read it as covering the token half.
        completeness: Completeness::known_wrong(
            "draws on every NONTOKEN creature-you-control ETB regardless of name. The nontoken \
             restriction is implemented (TargetFilter.is_nontoken, honoured by the ETB trigger \
             dispatch in rules/abilities.rs — PB-DX26 / OOS-DX3b-1, 2026-08-11). Still missing \
             the 'doesn't share a name with another creature you control or a creature card in \
             your graveyard' intervening-if: no name-uniqueness Condition variant exists \
             (re-verified 2026-08-11). Overdraws vs. the real card whenever two same-named \
             nontoken creatures are involved.",
        ),
        ..Default::default()
    }
}

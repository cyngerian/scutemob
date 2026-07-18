// Disciple of Freyalise // Garden of Freyalise — Modal DFC (Dominaria)
// Front: {3}{G}{G}{G} Creature — Elf Druid 3/3. When this creature enters, you may
//        sacrifice another creature. If you do, you gain X life and draw X cards,
//        where X is that creature's power.
// Back:  Garden of Freyalise — Land. As this land enters, you may pay 3 life. If you
//        don't, it enters tapped. {T}: Add {G}.
//
// Back face is fully authorable (mirrors Blood Crypt / Revitalizing Repast) and is
// implemented below. The front-face ETB trigger is NOT implemented — see the
// `completeness` note: the "another creature" restriction cannot currently be
// enforced by any sacrifice primitive.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("disciple-of-freyalise"),
        name: "Disciple of Freyalise".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 3,
            ..Default::default()
        }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "When this creature enters, you may sacrifice another creature. If you do, \
                      you gain X life and draw X cards, where X is that creature's power."
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // TODO: "When this creature enters, you may sacrifice another creature. If
            // you do, you gain X life and draw X cards, where X is that creature's
            // power." Would be Effect::MayPayThenEffect { cost:
            // Cost::Sacrifice(TargetFilter { has_card_type: Some(Creature), exclude_self:
            // true, .. }), payer: Controller, then: Sequence[GainLife{PowerOfSacrificed
            // Creature}, DrawCards{PowerOfSacrificedCreature}] }, but CONFIRMED (source
            // read 2026-07-17): `TargetFilter::exclude_self` is structurally unenforceable
            // here. `Cost::Sacrifice` payment (both the mandatory and `pay_optional_cost`
            // paths used by `MayPayThenEffect`) resolves eligible targets via
            // `eligible_sacrifice_targets` (effects/mod.rs:7346), which filters through
            // `matches_filter(chars: &Characteristics, filter: &TargetFilter)`
            // (effects/mod.rs:7941) — that function receives no ObjectId, so it has no way
            // to compare a candidate against `ctx.source` and can never honor
            // `exclude_self`. This means an implementation using this cost would let the
            // controller sacrifice Disciple of Freyalise itself to pay for its own
            // trigger, which oracle text ("another creature") forbids — wrong game state,
            // not merely a missing clause (W5 policy: do not ship this).
            //
            // Same confirmed gap blocks Commissar Severina Raine's "Sacrifice another
            // creature" activated cost and Korvold, Fae-Cursed King's "sacrifice another
            // permanent" ETB/attack trigger (see those defs). Fix needs either an
            // ObjectId-aware `matches_filter`/`eligible_sacrifice_targets`, or a dedicated
            // `Cost::SacrificeOther` variant that excludes the ability's source by
            // construction.
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Garden of Freyalise".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "As this land enters, you may pay 3 life. If you don't, it enters \
                          tapped.\n{T}: Add {G}."
                .to_string(),
            power: None,
            toughness: None,
            abilities: vec![
                // CR 614.1c: pay-3-life-or-enters-tapped self-replacement.
                AbilityDefinition::Replacement {
                    trigger: ReplacementTrigger::WouldEnterBattlefield {
                        filter: ObjectFilter::Any,
                    },
                    modification: ReplacementModification::EntersTappedUnlessPayLife(3),
                    is_self: true,
                    unless_condition: None,
                },
                // {T}: Add {G}.
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 0, 0, 1, 0),
                    },
                    timing_restriction: None,
                    targets: vec![],
                    activation_condition: None,
                    activation_zone: None,
                    once_per_turn: false,
                },
            ],
            color_indicator: None,
        }),
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::partial(
            "Back face (Garden of Freyalise) is fully implemented and correct. Front-face ETB \
             ('you may sacrifice another creature') is NOT implemented: CONFIRMED (source read \
             2026-07-17) that `TargetFilter::exclude_self` cannot be enforced by \
             `Cost::Sacrifice` in either the mandatory or `pay_optional_cost` (MayPayThenEffect) \
             path — `eligible_sacrifice_targets` (effects/mod.rs:7346) filters via \
             `matches_filter(chars, filter)` (effects/mod.rs:7941), which takes no ObjectId and \
             so cannot compare a candidate against ctx.source. Implementing this trigger today \
             would let the controller sacrifice Disciple of Freyalise itself, contradicting \
             oracle text ('another creature') — wrong game state per W5 policy, not shipped. Same \
             confirmed gap blocks commissar_severina_raine.rs and korvold_fae_cursed_king.rs. \
             Needs an ObjectId-aware exclude_self check in the sacrifice-cost path, or a \
             dedicated Cost::SacrificeOther variant.",
        ),
    }
}

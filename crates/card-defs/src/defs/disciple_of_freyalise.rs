// Disciple of Freyalise // Garden of Freyalise — Modal DFC (Dominaria)
// Front: {3}{G}{G}{G} Creature — Elf Druid 3/3. When this creature enters, you may
//        sacrifice another creature. If you do, you gain X life and draw X cards,
//        where X is that creature's power.
// Back:  Garden of Freyalise — Land. As this land enters, you may pay 3 life. If you
//        don't, it enters tapped. {T}: Add {G}.
//
// Back face is fully authorable (mirrors Blood Crypt / Revitalizing Repast) and is
// implemented below. The front-face ETB trigger is NOT implemented — see the
// `completeness` note. PB-EF1 closed the "another creature" blocker; the surviving
// blocker is that PowerOfSacrificedCreature is not captured in the optional-cost path.
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
            // "When this creature enters, you may sacrifice another creature. If you do,
            // you gain X life and draw X cards, where X is that creature's power."
            //
            // PB-EF1 (scutemob-99) CLOSED the exclude_self blocker: the optional-cost
            // sacrifice path (MayPayThenEffect → pay_optional_cost → eligible_sacrifice_targets)
            // now threads the ability source and honors TargetFilter.exclude_self (CR 109.1),
            // so "another creature" IS enforceable. BUT a SECOND blocker was found and this
            // card is NOT shipped:
            //
            //   EffectAmount::PowerOfSacrificedCreature reads ctx.sacrificed_creature_powers
            //   (effects/mod.rs), which is populated ONLY at the *activated-ability* sacrifice
            //   cost site (handle_activate_ability pushes sacrificed_lki_powers). The
            //   optional-cost path used by MayPayThenEffect (sacrifice_permanents_for_player)
            //   does NOT capture the sacrificed creature's power into ctx, so "gain X / draw X
            //   where X = that creature's power" would resolve X = 0 — wrong game state, not a
            //   missing clause (W5 policy: do not ship). Filed as PB-EF1 follow-up finding
            //   EF-EF1-A (see memory/primitives/ef-batch-plan-2026-07-17.md).
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
                    modes: None,
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
             ('you may sacrifice another creature; if you do, gain X life and draw X cards, X = \
             its power') is NOT implemented. PB-EF1 (scutemob-99) closed the original blocker — \
             the optional-cost sacrifice path now honors TargetFilter.exclude_self (CR 109.1), so \
             'another' is enforceable. The SURVIVING blocker: \
             EffectAmount::PowerOfSacrificedCreature reads ctx.sacrificed_creature_powers, \
             populated only at the activated-ability sacrifice-cost site — the MayPayThenEffect \
             optional-cost path (sacrifice_permanents_for_player) never captures the sacrificed \
             creature's power into ctx, so X would resolve to 0. Filed as PB-EF1 follow-up \
             EF-EF1-A. Not shipped (W5 policy: wrong game state, not a missing clause).",
        ),
    }
}

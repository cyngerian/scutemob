// Fable of the Mirror-Breaker // Reflection of Kiki-Jiki — {2}{R}, Enchantment
// — Saga (Transform)
// Front (Saga, chapters I-III):
//   I — Create a 2/2 red Goblin Shaman creature token with "Whenever this token
//       attacks, create a Treasure token."
//   II — You may discard up to two cards. If you do, draw that many cards.
//   III — Exile this Saga, then return it to the battlefield transformed under
//       your control.
// Back (Reflection of Kiki-Jiki, 2/2 Enchantment Creature — Goblin Shaman):
//   {1}, {T}: Create a token that's a copy of another target nonlegendary
//   creature you control, except it has haste. Sacrifice it at the beginning
//   of the next end step.
//
// PB-OS4 (OOS-EF5-3): chapter III is the primitive this PB adds — exiling the
// Saga then returning it transformed is CR 400.7 / 712.18 (a NEW object
// entering the battlefield already showing the Reflection of Kiki-Jiki face),
// wired here via `Effect::ExileSourceAndReturnTransformed`. Chapter III itself
// is fully expressible and tested; the card as a whole is `partial` because of
// two residuals unrelated to PB-OS4's primitive (see below).
use crate::cards::helpers::*;

fn goblin_shaman_token() -> TokenSpec {
    TokenSpec {
        name: "Goblin Shaman".to_string(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Goblin".to_string()), SubType("Shaman".to_string())]
            .into_iter()
            .collect(),
        colors: [Color::Red].into_iter().collect(),
        power: 2,
        toughness: 2,
        count: EffectAmount::Fixed(1),
        // TODO: "Whenever this token attacks, create a Treasure token." — TokenSpec
        // has no field for a triggered ability attached to a created token (only
        // `mana_abilities` and `activated_abilities`; no `triggered_abilities`).
        // ENGINE-BLOCKED: the token is created with correct P/T/color/subtypes but
        // without its own attack-trigger ability.
        ..Default::default()
    }
}

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fable-of-the-mirror-breaker"),
        name: "Fable of the Mirror-Breaker".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Enchantment], &["Saga"]),
        oracle_text: "(As this Saga enters and after your draw step, add a lore counter.)\nI — \
                      Create a 2/2 red Goblin Shaman creature token with \"Whenever this token \
                      attacks, create a Treasure token.\"\nII — You may discard up to two cards. \
                      If you do, draw that many cards.\nIII — Exile this Saga, then return it to \
                      the battlefield transformed under your control."
            .to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            // Chapter I: create the Goblin Shaman token (see the TODO on the token
            // spec above for the missing attack-trigger sub-ability).
            AbilityDefinition::SagaChapter {
                chapter: 1,
                effect: Effect::CreateToken {
                    spec: goblin_shaman_token(),
                },
                targets: vec![],
            },
            // Chapter II: "You may discard up to two cards. If you do, draw that many
            // cards." TODO: ENGINE-BLOCKED -- no DSL primitive for an optional,
            // player-chosen "discard up to N" whose count then drives a matching draw.
            // `Effect::DiscardCards` only supports a fixed/dynamically-resolved count,
            // not a bounded player choice; `Effect::WheelHand` only disposes of the
            // WHOLE hand (Wheel of Fortune / Windfall family), not "up to two". Left
            // as `Effect::Nothing` rather than force-fit a wrong-game-state stub.
            AbilityDefinition::SagaChapter {
                chapter: 2,
                effect: Effect::Nothing,
                targets: vec![],
            },
            // Chapter III: "Exile this Saga, then return it to the battlefield
            // transformed under your control." CR 400.7 / 712.18 -- the returned
            // permanent (Reflection of Kiki-Jiki) is a NEW object, not an in-place
            // flip. This is the PB-OS4 primitive (OOS-EF5-3).
            AbilityDefinition::SagaChapter {
                chapter: 3,
                effect: Effect::ExileSourceAndReturnTransformed,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Reflection of Kiki-Jiki".to_string(),
            mana_cost: None,
            types: types_sub(
                &[CardType::Enchantment, CardType::Creature],
                &["Goblin", "Shaman"],
            ),
            oracle_text: "{1}, {T}: Create a token that's a copy of another target nonlegendary \
                          creature you control, except it has haste. Sacrifice it at the \
                          beginning of the next end step."
                .to_string(),
            power: Some(2),
            toughness: Some(2),
            abilities: vec![
                // "{1}, {T}: Create a token that's a copy of another target nonlegendary
                // creature you control, except it has haste. Sacrifice it at the
                // beginning of the next end step." Same shape as Kiki-Jiki, Mirror
                // Breaker's own activated ability (kiki_jiki_mirror_breaker.rs),
                // including its known-wrong residual: `TargetFilter` has no
                // "nonlegendary" exclusion (only `legendary: bool` = must-BE-legendary),
                // so an illegal legendary target is not rejected. `exclude_self: true`
                // correctly encodes "another".
                AbilityDefinition::Activated {
                    cost: Cost::Sequence(vec![
                        Cost::Mana(ManaCost {
                            generic: 1,
                            ..Default::default()
                        }),
                        Cost::Tap,
                    ]),
                    effect: Effect::CreateTokenCopy {
                        source: EffectTarget::DeclaredTarget { index: 0 },
                        enters_tapped_and_attacking: false,
                        except_not_legendary: false,
                        gains_haste: true,
                        delayed_action: Some((
                            crate::state::stubs::DelayedTriggerTiming::AtNextEndStep,
                            crate::state::stubs::DelayedTriggerAction::SacrificeObject,
                        )),
                    },
                    timing_restriction: Some(TimingRestriction::SorcerySpeed),
                    targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::You,
                        exclude_self: true,
                        ..Default::default()
                    })],
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
            "chapter I's Goblin Shaman token is created with correct P/T/color/subtypes but \
             without its own \"whenever this token attacks, create a Treasure token\" ability -- \
             TokenSpec has no field for a triggered ability attached to a created token. Chapter \
             II (\"You may discard up to two cards. If you do, draw that many cards.\") is \
             Effect::Nothing -- no DSL primitive for a bounded optional discard whose count \
             drives a matching draw (DiscardCards has no player-choice bound; WheelHand only \
             disposes of the whole hand). Chapter III (the PB-OS4 primitive this card exists to \
             exercise -- Effect::ExileSourceAndReturnTransformed, CR 400.7/712.18) IS fully wired \
             and correct. The back face's activated ability also inherits Kiki-Jiki, Mirror \
             Breaker's known-wrong gap (TargetFilter has no \"nonlegendary\" exclusion).",
        ),
    }
}

// Deathrite Shaman — {B/G}, Creature — Elf Shaman 1/2
// {T}: Exile target land card from a graveyard. Add one mana of any color.
// {B}, {T}: Exile target instant or sorcery card from a graveyard. Each opponent loses 2 life.
// {G}, {T}: Exile target creature card from a graveyard. You gain 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("deathrite-shaman"),
        name: "Deathrite Shaman".to_string(),
        mana_cost: Some(ManaCost {
            hybrid: vec![HybridMana::ColorColor(ManaColor::Black, ManaColor::Green)],
            ..Default::default()
        }),
        types: creature_types(&["Elf", "Shaman"]),
        oracle_text: "{T}: Exile target land card from a graveyard. Add one mana of any \
                      color.\n{B}, {T}: Exile target instant or sorcery card from a graveyard. \
                      Each opponent loses 2 life.\n{G}, {T}: Exile target creature card from a \
                      graveyard. You gain 2 life."
            .to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // {T}: Exile target land card from a graveyard. Add one mana of any color.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Sequence(vec![
                    Effect::ExileObject {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::AddManaAnyColor {
                        player: PlayerTarget::Controller,
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCardInGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Land),
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // {B}, {T}: Exile target instant or sorcery card from a graveyard.
            // Each opponent loses 2 life.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        black: 1,
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::Sequence(vec![
                    Effect::ExileObject {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::ForEach {
                        over: ForEachTarget::EachOpponent,
                        effect: Box::new(Effect::LoseLife {
                            player: PlayerTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(2),
                        }),
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCardInGraveyard(TargetFilter {
                    has_card_types: vec![CardType::Instant, CardType::Sorcery],
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // {G}, {T}: Exile target creature card from a graveyard. You gain 2 life.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        green: 1,
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::Sequence(vec![
                    Effect::ExileObject {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(2),
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCardInGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::known_wrong(
            "PB-EF12 (EF-W-PB2-3) re-triage: this card is NOT restored, contrary to an earlier \
             pass in this same task that eyeballed it as a plain served ability and was wrong. \
             The first ability ('{T}: Exile target land card from a graveyard. Add one mana of \
             any color.') has a TARGET (TargetCardInGraveyard) — CR 605.1a: a mana ability \
             'doesn't require a target'. `mana_ability_lowering` (testing/replay_harness.rs) \
             checks `targets.is_empty()` FIRST and returns None for any non-empty target list, so \
             this whole ability is never lowered into a ManaAbility regardless of its \
             Effect::AddManaAnyColor payload — it stays a stack-using Activated ability, and the \
             nested Effect::AddManaAnyColor (inside a Sequence with ExileObject) resolves through \
             execute_effect's stack-resolution arm, which still adds ManaColor::Colorless \
             unconditionally. Caught by the refined effect_choose_gate \
             (no_complete_def_uses_an_any_color_mana_stub), which is exactly why that gate is \
             programmatic rather than eyeballed. The other two abilities ({B}/{G} \
             exile+drain/gain) are correctly implemented and unaffected.",
        ),
        ..Default::default()
    }
}

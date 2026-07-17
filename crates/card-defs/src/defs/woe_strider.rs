// Woe Strider — {2}{B}, Creature — Horror 3/2
// When this creature enters, create a 0/1 white Goat creature token.
// Sacrifice another creature: Scry 1.
// Escape—{3}{B}{B}, Exile four other cards from your graveyard.
// This creature escapes with two +1/+1 counters on it.
//
// TODO: "This creature escapes with two +1/+1 counters" — needs Escape ETB counter
//   replacement effect (the counters are only added when cast via Escape, not normally).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("woe-strider"),
        name: "Woe Strider".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            ..Default::default()
        }),
        types: creature_types(&["Horror"]),
        oracle_text: "When this creature enters, create a 0/1 white Goat creature \
                      token.\nSacrifice another creature: Scry 1.\nEscape—{3}{B}{B}, Exile four \
                      other cards from your graveyard. (You may cast this card from your \
                      graveyard for its escape cost.)\nThis creature escapes with two +1/+1 \
                      counters on it."
            .to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // ETB: create a 0/1 white Goat token
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Goat".to_string(),
                        power: 0,
                        toughness: 1,
                        colors: [Color::White].into_iter().collect(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Goat".to_string())].into_iter().collect(),
                        count: EffectAmount::Fixed(1),
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // Sacrifice another creature: Scry 1
            AbilityDefinition::Activated {
                cost: Cost::Sacrifice(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                }),
                effect: Effect::Scry {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // Escape keyword marker
            AbilityDefinition::Keyword(KeywordAbility::Escape),
            // Escape—{3}{B}{B}, Exile four other cards
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Escape,
                cost: ManaCost {
                    generic: 3,
                    black: 2,
                    ..Default::default()
                },
                details: Some(AltCastDetails::Escape { exile_count: 4 }),
            },
        ],
        completeness: Completeness::partial(
            "Two items. (1) Blocker shipped: add AbilityDefinition::EscapeWithCounter \
             (card_definition.rs:521; wired at resolution.rs:853, tested at \
             tests/mechanics_e_l/escape.rs:141) for 'escapes with two +1/+1 counters'. (2) Still \
             blocked / currently wrong: 'Sacrifice another creature: Scry 1' uses Cost::Sacrifice \
             with a bare creature filter, which cannot exclude the source — Woe Strider can \
             sacrifice itself. Same exclude-self gap as wight_of_the_reliquary.rs / \
             vampire_gourmand.rs, which omit the ability instead.",
        ),
        ..Default::default()
    }
}

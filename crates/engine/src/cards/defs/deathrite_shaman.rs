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
        oracle_text: "{T}: Exile target land card from a graveyard. Add one mana of any color.\n{B}, {T}: Exile target instant or sorcery card from a graveyard. Each opponent loses 2 life.\n{G}, {T}: Exile target creature card from a graveyard. You gain 2 life.".to_string(),
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
            },
            // {B}, {T}: Exile target instant or sorcery card from a graveyard.
            // Each opponent loses 2 life.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { black: 1, ..Default::default() }),
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
            },
            // {G}, {T}: Exile target creature card from a graveyard. You gain 2 life.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { green: 1, ..Default::default() }),
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
            },
        ],
        ..Default::default()
    }
}

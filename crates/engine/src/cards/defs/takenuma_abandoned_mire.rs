// Takenuma, Abandoned Mire — Legendary Land, {T}: Add {B}; Channel — mill + return from GY.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("takenuma-abandoned-mire"),
        name: "Takenuma, Abandoned Mire".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {B}.\nChannel — {3}{B}, Discard this card: Mill three cards, then return a creature or planeswalker card from your graveyard to your hand. This ability costs {1} less to activate for each legendary creature you control.".to_string(),
        abilities: vec![
            // {T}: Add {B}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // Channel — {3}{B}, Discard this card: Mill 3, then return creature/planeswalker
            // from graveyard to hand. Deterministic fallback picks highest-ObjectId matching card.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 3, black: 1, ..Default::default() }),
                    Cost::DiscardSelf,
                ]),
                effect: Effect::Sequence(vec![
                    Effect::MillCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(3),
                    },
                    // CR 701.13: "return a creature or planeswalker card from your graveyard to
                    // your hand" — uses has_card_types (OR semantics) for multi-type filter.
                    Effect::MoveZone {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                        controller_override: None,
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_types: vec![CardType::Creature, CardType::Planeswalker],
                    ..Default::default()
                })],
                activation_condition: None,
            },
        ],
        // CR 602.2b + 601.2f: Channel ability (index 0) costs {1} less per legendary creature.
        activated_ability_cost_reductions: vec![(
            0,
            SelfActivatedCostReduction::PerPermanent {
                per: 1,
                filter: TargetFilter {
                    legendary: true,
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                },
                controller: PlayerTarget::Controller,
            },
        )],
        ..Default::default()
    }
}

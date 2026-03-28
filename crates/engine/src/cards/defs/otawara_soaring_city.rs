// Otawara, Soaring City — Legendary Land, {T}: Add {U}; Channel — bounce target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("otawara-soaring-city"),
        name: "Otawara, Soaring City".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {U}.\nChannel — {3}{U}, Discard this card: Return target artifact, creature, enchantment, or planeswalker to its owner's hand. This ability costs {1} less to activate for each legendary creature you control.".to_string(),
        abilities: vec![
            // {T}: Add {U}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // Channel — {3}{U}, Discard this card: Return target non-land permanent to
            // owner's hand. "artifact, creature, enchantment, or planeswalker" = any permanent
            // that is not a land. Using non_land filter to approximate (excludes lands).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 3, blue: 1, ..Default::default() }),
                    Cost::DiscardSelf,
                ]),
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand { owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })) },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    non_land: true,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
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

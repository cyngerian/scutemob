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
            },
            // Channel — {3}{U}, Discard this card: Return target non-land permanent to
            // owner's hand. "artifact, creature, enchantment, or planeswalker" = any permanent
            // that is not a land. Using non_land filter to approximate (excludes lands).
            // TODO: Cost reduction — {1} less per legendary creature you control.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 3, blue: 1, ..Default::default() }),
                    Cost::DiscardSelf,
                ]),
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand { owner: PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })) },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    non_land: true,
                    ..Default::default()
                })],
            },
        ],
        ..Default::default()
    }
}

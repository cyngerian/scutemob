// Reassembling Skeleton — {1}{B}, Creature — Skeleton Warrior 1/1.
// "{1}{B}: Return Reassembling Skeleton from your graveyard to the
// battlefield tapped."
//
// CR 602.2 / PB-35: Graveyard-activated ability implemented via
// ActivationZone::Graveyard. Returns to battlefield tapped.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reassembling-skeleton"),
        name: "Reassembling Skeleton".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Skeleton", "Warrior"]),
        oracle_text: "{1}{B}: Return Reassembling Skeleton from your graveyard to the battlefield tapped.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 602.2 / PB-35: "{1}{B}: Return this card from your graveyard to the
            // battlefield tapped." — activated from the graveyard zone.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, black: 1, ..Default::default() }),
                effect: Effect::MoveZone {
                    target: EffectTarget::Source,
                    to: ZoneTarget::Battlefield { tapped: true },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: Some(ActivationZone::Graveyard),
            },
        ],
        ..Default::default()
    }
}

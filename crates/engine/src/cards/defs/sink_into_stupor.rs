// Sink into Stupor // Soporific Springs — {1}{U}{U} Instant // Land (MDFC)
// Oracle: "Return target spell or nonland permanent an opponent controls to its owner's hand."
// Note: TargetSpell OR nonland permanent not expressible as single target requirement.
// Approximated as nonland permanent an opponent controls (spell-targeting omitted).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sink-into-stupor"),
        name: "Sink into Stupor // Soporific Springs".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Return target spell or nonland permanent an opponent controls to its owner's hand.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::MoveZone {
                target: EffectTarget::DeclaredTarget { index: 0 },
                to: ZoneTarget::Hand {
                    owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget {
                        index: 0,
                    })),
                },
                controller_override: None,
            },
            // TODO: Should also target spells on the stack — no combined target variant.
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                non_land: true,
                controller: TargetController::Opponent,
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

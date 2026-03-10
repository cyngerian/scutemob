// Assassin's Trophy — {B}{G} Instant; destroy target nonland permanent.
// Its controller may search their library for a basic land card, put it onto
// the battlefield, then shuffle.
// TODO: DSL gap — "its controller searches for a basic land" requires a
// targeted search effect for the target's controller. Only the destroy portion
// is implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("assassins-trophy"),
        name: "Assassin's Trophy".to_string(),
        mana_cost: Some(ManaCost { black: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target nonland permanent. Its controller may search their library for a basic land card, put it onto the battlefield, then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
            },
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                non_land: true,
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

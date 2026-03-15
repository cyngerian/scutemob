// Boseiju, Who Endures — Legendary Land
// {T}: Add {G}. Channel — destroy target artifact/enchantment/nonbasic land an opponent controls.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boseiju-who-endures"),
        name: "Boseiju, Who Endures".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {G}.\nChannel — {1}{G}, Discard this card: Destroy target artifact, enchantment, or nonbasic land an opponent controls. That player may search their library for a land card with a basic land type, put it onto the battlefield, then shuffle. This ability costs {1} less to activate for each legendary creature you control.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // Channel — {1}{G}, Discard this card: Destroy target permanent.
            // TODO: Target filter should restrict to "artifact, enchantment, or nonbasic land
            //       an opponent controls". Using TargetPermanent as approximation.
            // TODO: "That player may search their library for a land card with a basic land type,
            //       put it onto the battlefield, then shuffle" — opponent search not expressible.
            // TODO: Cost reduction — {1} less per legendary creature you control.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, green: 1, ..Default::default() }),
                    Cost::DiscardSelf,
                ]),
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetPermanent],
            },
        ],
        ..Default::default()
    }
}

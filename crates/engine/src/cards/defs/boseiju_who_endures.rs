// Boseiju, Who Endures — Legendary Land
// {T}: Add {G}.
// Channel — {1}{G}, Discard this card: Destroy target artifact, enchantment, or nonbasic
//   land an opponent controls. That player may search for land with basic land type,
//   put onto battlefield, shuffle. This ability costs {1} less per legendary creature you control.
// TODO: Target filter should restrict to "artifact, enchantment, or nonbasic land an opponent
//   controls" — using TargetPermanent as approximation.
// TODO: Cost reduction — {1} less per legendary creature you control.
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
            // Channel — {1}{G}, Discard: Destroy target + opponent searches.
            // TODO: Target filter needs "artifact, enchantment, or nonbasic land an opponent controls"
            // TODO: Cost reduction per legendary creature
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 1,
                        green: 1,
                        ..Default::default()
                    }),
                    Cost::DiscardSelf,
                ]),
                effect: Effect::Sequence(vec![
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::SearchLibrary {
                        player: PlayerTarget::ControllerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            basic: true,
                            ..Default::default()
                        },
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: false },
                    },
                    Effect::Shuffle {
                        player: PlayerTarget::ControllerOf(Box::new(
                            EffectTarget::DeclaredTarget { index: 0 },
                        )),
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetPermanent],
            },
        ],
        ..Default::default()
    }
}

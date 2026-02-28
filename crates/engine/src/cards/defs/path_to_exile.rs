// 21. Path to Exile — {W}, Instant; exile target creature, its controller may
// search for a basic land and put it into play tapped.
// CR 701.19: "may search" is modelled via MayPayOrElse with zero cost.
// M9.4 deterministic fallback: payer does not pay → or_else (search) fires.
// The exiled creature's controller is the payer (ControllerOf target).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("path-to-exile"),
        name: "Path to Exile".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Exile target creature. Its controller may search their library for a basic land card, put that card onto the battlefield tapped, then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::ExileObject {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                // "May search" — modelled as MayPayOrElse with zero cost.
                // or_else = search (fires when player declines to "pay" the
                // zero cost, i.e. chooses NOT to search in interactive play).
                // Deterministic fallback: always fires or_else (always searches).
                Effect::MayPayOrElse {
                    cost: Cost::Mana(
                        ManaCost { ..Default::default() }
                    ),
                    payer: PlayerTarget::ControllerOf(Box::new(
                        EffectTarget::DeclaredTarget { index: 0 },
                    )),
                    or_else: Box::new(Effect::Sequence(vec![
                        Effect::SearchLibrary {
                            player: PlayerTarget::ControllerOf(Box::new(
                                EffectTarget::DeclaredTarget { index: 0 },
                            )),
                            filter: basic_land_filter(),
                            reveal: false,
                            destination: ZoneTarget::Battlefield {
                                tapped: true,
                            },
                        },
                        Effect::Shuffle {
                            player: PlayerTarget::ControllerOf(Box::new(
                                EffectTarget::DeclaredTarget { index: 0 },
                            )),
                        },
                    ])),
                },
            ]),
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

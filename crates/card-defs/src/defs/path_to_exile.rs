// 21. Path to Exile — {W}, Instant; exile target creature, its controller may
// search for a basic land and put it into play tapped.
//
// SR-33: the search is NOT optional here, and previously said it was. `MayPayOrElse`
// (effects/mod.rs) discards `cost` and `payer` and unconditionally executes `or_else`,
// so the controller of the exiled creature always searches and always ramps — they are
// never offered the choice to decline. That is a real game-state deviation (declining is
// often correct play), not a deterministic-but-legal shortcut, so this def is
// `known_wrong` until interactive choice exists. It was `Complete`, and the
// `completeness_deviation_scan` ALLOWLIST justified it as "a faithful encoding of the
// optional search" — false; the entry has been removed.
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
                // or_else = search. NOTE (SR-33): MayPayOrElse never collects the
                // payment and always runs or_else, so this ALWAYS searches. The
                // "may" is not expressible today; see the known_wrong marker below.
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
                            shuffle_before_placing: false,
                    also_search_graveyard: false,
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
        completeness: Completeness::known_wrong(
            "SR-33: the search always fires. `Effect::MayPayOrElse` discards its `cost` and \
             `payer` and unconditionally executes `or_else` (effects/mod.rs), so the exiled \
             creature's controller is never offered the option to decline and always gets the \
             basic land. Needs a general choice Command (M9+); until then the exile half is \
             correct and the ramp half is unconditional.",
        ),
        ..Default::default()
    }
}

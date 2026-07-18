// Mystic Sanctuary — ({T}: Add {U}.) This land enters tapped unless you control three or more
// other Islands. When this land enters untapped, you may put target instant or sorcery card from
// your graveyard on top of your library.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mystic-sanctuary"),
        name: "Mystic Sanctuary".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Island"]),
        oracle_text: "({T}: Add {U}.)\nThis land enters tapped unless you control three or more \
                      other Islands.\nWhen this land enters untapped, you may put target instant \
                      or sorcery card from your graveyard on top of your library."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::ControlAtLeastNOtherLandsWithSubtype {
                    count: 3,
                    subtype: SubType("Island".to_string()),
                }),
            },
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
                once_per_turn: false,
                modes: None,
            },
            // TODO: Triggered — When this land enters untapped, you may put target instant or sorcery card from your graveyard on top of your library.
        ],
        completeness: Completeness::partial(
            "Sole blocker: 'you may' has no correct expression — AbilityDefinition::Triggered has \
             no optional/may field and Effect::Choose is non-interactive (effects/mod.rs:3190 \
             always runs choices.first()). Everything else is available: WhenEntersBattlefield + \
             TargetCardInYourGraveyard(instant-or-sorcery) + MoveZone to LibraryPosition::Top, \
             with the 'enters untapped' gate via Effect::Conditional { condition: \
             Condition::SourceIsUntapped }; see mortuary_mire.rs for the analogous wiring. Same \
             'you may' blocker as mortuary_mire.",
        ),
        ..Default::default()
    }
}

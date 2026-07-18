// Witch's Cottage — ({T}: Add {B}.) This land enters tapped unless you control three or more
// other Swamps. When this land enters untapped, you may put target creature card from your
// graveyard on top of your library.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("witchs-cottage"),
        name: "Witch's Cottage".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Swamp"]),
        oracle_text: "({T}: Add {B}.)\nThis land enters tapped unless you control three or more \
                      other Swamps.\nWhen this land enters untapped, you may put target creature \
                      card from your graveyard on top of your library."
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
                    subtype: SubType("Swamp".to_string()),
                }),
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // TODO: Triggered — When this land enters untapped, you may put target creature card from your graveyard on top of your library.
        ],
        completeness: Completeness::partial(
            "Blocked on (a) no trigger condition for 'enters UNTAPPED' (WhenEntersBattlefield \
             fires regardless of tapped state; no intervening_if reads the source's own tapped \
             state) and (b) 'you may' has no interactive expression. TargetCardInYourGraveyard \
             and Effect::PutOnLibrary both exist — the effect half is not the blocker.",
        ),
        ..Default::default()
    }
}

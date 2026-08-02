// 112. Windbrisk Heights — Land; Hideaway 4; enters tapped; {T}: {W}; {W},{T}: play exiled card.
// CR 702.75: Hideaway 4 triggers on ETB: look at top 4, exile one face-down, put rest on bottom.
// CR 702.75b: older Hideaway cards errata'd to "Hideaway 4" + separate "enters tapped" line.
// The play condition ("attacked with 3+ creatures this turn") uses
// Condition::YouAttackedWithNOrMore(3), which reads PlayerState.attackers_declared_this_turn
// (see player.rs) — attack tracking exists and is wired here as the activation_condition.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("windbrisk-heights"),
        name: "Windbrisk Heights".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "Hideaway 4 (When this land enters, look at the top four cards of your \
                      library, exile one face down, then put the rest on the bottom in a random \
                      order.)\nThis land enters tapped.\n{T}: Add {W}.\n{W}, {T}: You may play \
                      the exiled card without paying its mana cost if you attacked with three or \
                      more creatures this turn."
            .to_string(),
        abilities: vec![
            // CR 702.75: Hideaway 4 — ETB trigger wired via KeywordAbility::Hideaway(4).
            AbilityDefinition::Keyword(KeywordAbility::Hideaway(4)),
            // CR 614.1c: self-replacement — this land enters the battlefield tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // {T}: Add {W} (no Plains subtype on the printed card; ability is explicit).
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // {W}, {T}: Play the exiled card without paying its mana cost, if you attacked
            // with three or more creatures this turn (CR ruling 2007-10-01: any point in the
            // turn, counted by distinct creatures declared as attackers).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        white: 1,
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::PlayExiledCard,
                timing_restriction: None,
                targets: vec![],
                activation_condition: Some(Condition::YouAttackedWithNOrMore(3)),
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        ..Default::default()
    }
}

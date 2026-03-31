// Elvish Reclaimer — {G} Creature — Elf Warrior 1/2.
// "This creature gets +2/+2 as long as there are three or more land cards in your graveyard."
// "{2}, {T}, Sacrifice a land: Search your library for a land card, put it onto
// the battlefield tapped, then shuffle."
//
// The static +2/+2 ability requires Condition::CardsInGraveyardAtLeast { card_type: Land, min: 3 },
// which does not exist in the current DSL (no graveyard-count condition for static P/T buffs).
// The activated land-search ability is fully expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elvish-reclaimer"),
        name: "Elvish Reclaimer".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Warrior"]),
        oracle_text: "This creature gets +2/+2 as long as there are three or more land cards in your graveyard.\n{2}, {T}, Sacrifice a land: Search your library for a land card, put it onto the battlefield tapped, then shuffle.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // TODO: "gets +2/+2 as long as there are three or more land cards in your graveyard"
            // Needs Condition/EffectFilter checking controller's graveyard card count by type.
            // No current DSL variant for "graveyard contains N or more cards of type X" as
            // a static continuous effect condition.

            // {2}, {T}, Sacrifice a land: Search your library for a land card,
            // put it onto the battlefield tapped, then shuffle.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::Tap,
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        },
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: true },
                        shuffle_before_placing: false,
                        also_search_graveyard: false,
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

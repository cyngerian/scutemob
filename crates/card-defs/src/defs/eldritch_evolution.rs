// Eldritch Evolution — {1}{G}{G}, Sorcery
// "As an additional cost to cast this spell, sacrifice a creature.
//  Search your library for a creature card with mana value X or less, where X is 2
//  plus the sacrificed creature's mana value. Put that card onto the battlefield,
//  then shuffle. Exile Eldritch Evolution."
//
// CR 118.8: Mandatory additional sacrifice cost at cast time.
// CR 608.2h/202.3: X is 2 plus the LKI mana value of the sacrificed creature
// (captured BEFORE move_object_to_zone at the spell-additional-cost sacrifice site,
// PB-EF10). The runtime cap lives on TargetFilter.max_cmc_amount (a runtime UPPER
// BOUND — "or less" — honored only by the SearchLibrary executor, not by
// matches_filter). "then shuffle" is modeled explicitly with Effect::Shuffle
// (mirrors Harrow's pattern) rather than relying on the SearchLibrary executor's
// shuffle_before_placing flag, which only shuffles BEFORE placing.
// "Exile Eldritch Evolution" -- self_exile_on_resolution replaces the normal
// graveyard move after the spell resolves.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eldritch-evolution"),
        name: "Eldritch Evolution".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a creature.\nSearch \
                      your library for a creature card with mana value X or less, where X is 2 \
                      plus the sacrificed creature's mana value. Put that card onto the \
                      battlefield, then shuffle. Exile Eldritch Evolution."
            .to_string(),
        // CR 118.8: Mandatory sacrifice of a creature as additional cost.
        spell_additional_costs: vec![SpellAdditionalCost::SacrificeCreature],
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        max_cmc_amount: Some(Box::new(EffectAmount::Sum(
                            Box::new(EffectAmount::Fixed(2)),
                            Box::new(EffectAmount::ManaValueOfSacrificedCreature),
                        ))),
                        ..Default::default()
                    },
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: false },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::Shuffle {
                    player: PlayerTarget::Controller,
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        // "Exile Eldritch Evolution." -- self-exile on resolution.
        self_exile_on_resolution: true,
        ..Default::default()
    }
}

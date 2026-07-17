// Fabled Passage — Land; {T}, sacrifice: search for basic land (tapped), then if 4+ lands untap it.
// TODO: The conditional untap ("if you control four or more lands, untap that land") requires
// (a) a Condition::YouControlNOrMoreLands variant (not in DSL) and
// (b) an Effect::UntapLastSearchedCard or similar way to reference the land just placed.
// The base search-and-tapped effect is implemented; the untap bonus is omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fabled-passage"),
        name: "Fabled Passage".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}, Sacrifice this land: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle. Then if you control four or more lands, untap that land.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sequence(vec![
                Cost::Tap,
                Cost::SacrificeSelf,
            ]),
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: basic_land_filter(),
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
        }],
        completeness: Completeness::partial("partial(\"'Then if you control four or more lands, untap that land' — blocked ONLY on referencing the searched land. Effect::SearchLibrary does not set ctx.last_created_permanent (written solely by CreateToken/Manifest/Cloak — effects/mod.rs:714/3637/3687/4893), so EffectTarget::LastCreatedPermanent has no referent. The condition half is NOT a gap: Condition::YouControlNOrMoreWithFilter { count: 4, filter: { has_card_type: Land, controller: You } } exists (card_definition.rs:3571). Needs SearchLibrary to publish the placed card into EffectContext. Search-and-tapped clause implemented.\")"),
        ..Default::default()
    }
}

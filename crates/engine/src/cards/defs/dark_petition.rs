// Dark Petition — {3}{B}{B}, Sorcery; search for a card to hand, then shuffle.
// Spell mastery — if 2+ instant/sorcery cards in graveyard, add {B}{B}{B}.
// TODO: Spell mastery conditional mana bonus ({B}{B}{B} if 2+ instant/sorcery in
// graveyard) cannot be expressed. No Condition variant for "N or more instants and/or
// sorceries in your graveyard" exists in the DSL. The base search effect is implemented;
// the bonus mana is omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dark-petition"),
        name: "Dark Petition".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text:
            "Search your library for a card, put that card into your hand, then shuffle.\n\
             Spell mastery — If there are two or more instant and/or sorcery cards in your \
             graveyard, add {B}{B}{B}."
                .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter::default(),
                    reveal: false,
                    destination: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::Shuffle { player: PlayerTarget::Controller },
                // TODO: Condition::SpellMastery (2+ instant/sorcery in graveyard) not in DSL.
                // If condition is met, add {B}{B}{B} to controller's mana pool.
                // Omitting bonus mana — wrong for spell mastery games but base tutor is correct.
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

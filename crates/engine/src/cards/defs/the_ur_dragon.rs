// The Ur-Dragon — {4}{W}{U}{B}{R}{G}, Legendary Creature — Dragon Avatar 10/10
// Eminence — As long as The Ur-Dragon is in the command zone or on the battlefield,
// other Dragon spells you cast cost {1} less to cast.
// Flying
// Whenever one or more Dragons you control attack, draw that many cards, then you may put
// a permanent card from your hand onto the battlefield.
// TODO: attack trigger — draw X cards where X = number of attacking Dragons,
//       then put a permanent from hand onto the battlefield
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-ur-dragon"),
        name: "The Ur-Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 4, white: 1, blue: 1, black: 1, red: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Avatar"],
        ),
        oracle_text: "Eminence — As long as The Ur-Dragon is in the command zone or on the battlefield, other Dragon spells you cast cost {1} less to cast.\nFlying\nWhenever one or more Dragons you control attack, draw that many cards, then you may put a permanent card from your hand onto the battlefield.".to_string(),
        power: Some(10),
        toughness: Some(10),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: attack trigger — draw X cards where X = number of attacking Dragons,
            //       then put a permanent from hand onto the battlefield
        ],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasSubtype(SubType("Dragon".to_string())),
            scope: CostModifierScope::Controller,
            eminence: true,
        }],
        ..Default::default()
    }
}

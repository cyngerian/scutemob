// Golgari Grave-Troll — {4G}, Creature — Troll Skeleton 0/0
// This creature enters with a +1/+1 counter on it for each creature card in your graveyard.
// {1}, Remove a +1/+1 counter from this creature: Regenerate this creature.
// Dredge 6
//
// Note: ETB counter placement (one per creature card in graveyard) deferred —
// needs a TriggeredEffect that counts graveyard contents at resolution.
//
// CR 702.52a: Dredge N — if you would draw a card, you may instead mill N cards
// and return this card from your graveyard to your hand. Functions only while
// this card is in the graveyard. Requires >= N cards in library (CR 702.52b).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("golgari-grave-troll"),
        name: "Golgari Grave-Troll".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 1, ..Default::default() }),
        types: creature_types(&["Troll", "Skeleton"]),
        oracle_text: "This creature enters with a +1/+1 counter on it for each creature card in your graveyard.\n{1}, Remove a +1/+1 counter from this creature: Regenerate this creature.\nDredge 6 (If you would draw a card, you may mill six cards instead. If you do, return this card from your graveyard to your hand.)"
            .to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            // CR 702.52a: Dredge 6 marker.
            AbilityDefinition::Keyword(KeywordAbility::Dredge(6)),
            // CR 602.2: {1}, Remove a +1/+1 counter from this creature: Regenerate.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::RemoveCounter { counter: CounterType::PlusOnePlusOne, count: 1 },
                ]),
                effect: Effect::Regenerate { target: EffectTarget::Source },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
    }
}

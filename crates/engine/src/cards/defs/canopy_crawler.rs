// Canopy Crawler — {3}{G}, Creature — Beast 2/2; Amplify 1; {T}: target creature
// gets +1/+1 until end of turn for each +1/+1 counter on this creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("canopy-crawler"),
        name: "Canopy Crawler".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: creature_types(&["Beast"]),
        oracle_text: "Amplify 1 (As this creature enters, put a +1/+1 counter on it for each Beast card you reveal in your hand.)\n{T}: Target creature gets +1/+1 until end of turn for each +1/+1 counter on Canopy Crawler.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Amplify(1)),
            // TODO: activated ability — {T}: target creature gets +1/+1 until end of turn
            // for each +1/+1 counter on this creature. Requires EffectAmount::CountersOnSelf
            // or similar variant (not yet in DSL).
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
    }
}

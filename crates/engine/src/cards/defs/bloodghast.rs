// Bloodghast — {B}{B}, Creature — Vampire Spirit 2/1.
// Can't block; has haste if opponent at 10 or less life (conditional static);
// Landfall — whenever a land you control enters, may return from graveyard.
//
// CR 604.2 / CR 613.1f (Layer 6): "This creature has haste as long as an opponent
// has 10 or less life." Implemented as a conditional static with Condition::OpponentLifeAtMost(10).
//
// TODO: "This creature can't block." — KeywordAbility::CantBlock does not exist in the DSL.
// The "can't block" restriction must be enforced via Decayed keyword or a future CantBlock
// keyword. Left as TODO; this is a DSL gap (PB-25 scope).
//
// TODO: Landfall — whenever a land you control enters, you may return this card from your
// graveyard to the battlefield. Triggered ability from the graveyard zone is not yet
// supported in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodghast"),
        name: "Bloodghast".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Spirit"]),
        oracle_text: "This creature can't block.\nThis creature has haste as long as an opponent has 10 or less life.\nLandfall \u{2014} Whenever a land you control enters, you may return this card from your graveyard to the battlefield.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            // CR 604.2 / CR 613.1f (Layer 6): "This creature has haste as long as an opponent
            // has 10 or less life." Haste is granted conditionally when any living opponent
            // has a life total of 10 or below.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                    filter: EffectFilter::Source,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::OpponentLifeAtMost(10)),
                },
            },
            // TODO: "This creature can't block." — KeywordAbility::CantBlock does not exist.
            // TODO: Landfall trigger from graveyard zone not in DSL.
        ],
        ..Default::default()
    }
}

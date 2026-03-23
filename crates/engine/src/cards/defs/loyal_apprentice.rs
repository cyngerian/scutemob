// Loyal Apprentice — {1}{R}, Creature — Human Artificer 2/1
// Haste
// Lieutenant — At the beginning of combat on your turn, if you control your commander,
// create a 1/1 colorless Thopter artifact creature token with flying. That token gains
// haste until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("loyal-apprentice"),
        name: "Loyal Apprentice".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Human", "Artificer"]),
        oracle_text: "Haste\nLieutenant — At the beginning of combat on your turn, if you control your commander, create a 1/1 colorless Thopter artifact creature token with flying. That token gains haste until end of turn.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: Lieutenant ability — requires intervening-if Condition::YouControlYourCommander
            //   which does not exist in DSL. Also token needs haste granted after creation
            //   (Effect::Sequence with CreateToken then ApplyContinuousEffect targeting the
            //   newly created token is not supported). W5 policy: no approximation.
        ],
        ..Default::default()
    }
}

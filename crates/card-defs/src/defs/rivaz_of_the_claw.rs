// Rivaz of the Claw
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rivaz-of-the-claw"),
        name: "Rivaz of the Claw".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 1,
            red: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Lizard", "Warlock"],
        ),
        oracle_text: "Menace\n{T}: Add two mana in any combination of colors. Spend this mana \
                      only to cast Dragon creature spells.\nOnce during each of your turns, you \
                      may cast a Dragon creature spell from your graveyard.\nWhenever you cast a \
                      Dragon creature spell from your graveyard, it gains \"When this creature \
                      dies, exile it.\""
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // TODO: DSL gap — restricted mana ability (Dragon spells only) + graveyard
            // casting permission + death-exile replacement grant. Multiple DSL gaps.
        ],
        completeness: Completeness::partial(
            "Menace only. ManaRestriction (Dragon-spells-only mana) and StaticPlayFromGraveyard / \
             CastSelfFromGraveyard both EXIST — those clauses of the old note were stale. Real \
             remaining blocker: granting a graveyard-cast Dragon spell the delayed 'When this \
             creature dies, exile it' ability, and the 'once during each of your turns' cast \
             restriction.",
        ),
        ..Default::default()
    }
}

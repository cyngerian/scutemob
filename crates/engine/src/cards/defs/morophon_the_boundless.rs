// Morophon, the Boundless — {7}, Legendary Creature — Shapeshifter 6/6
// Changeling (This card is every creature type.)
// As Morophon enters, choose a creature type.
// Spells of the chosen type you cast cost {W}{U}{B}{R}{G} less to cast.
//   This effect reduces only the amount of colored mana you pay.
// Other creatures you control of the chosen type get +1/+1.
//
// TODO: Colored mana cost reduction ({W}{U}{B}{R}{G} less) — SpellCostModifier only supports
//   generic mana changes. Morophon's reduction removes colored mana, which needs a new mechanism.
// TODO: SpellCostFilter for chosen creature type — no HasChosenSubtype variant exists.
// TODO: "Other creatures of the chosen type get +1/+1" — needs EffectFilter::ChosenSubtype
//   for the ContinuousEffectDef filter.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("morophon-the-boundless"),
        name: "Morophon, the Boundless".to_string(),
        mana_cost: Some(ManaCost { generic: 7, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Shapeshifter"],
        ),
        oracle_text: "Changeling (This card is every creature type.)\nAs Morophon enters, choose a creature type.\nSpells of the chosen type you cast cost {W}{U}{B}{R}{G} less to cast. This effect reduces only the amount of colored mana you pay.\nOther creatures you control of the chosen type get +1/+1.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Changeling),
            // "As Morophon enters, choose a creature type" — self-replacement (CR 614.1c)
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ChooseCreatureType(SubType("Human".to_string())),
                is_self: true,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}

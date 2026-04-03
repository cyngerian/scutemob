// Morophon, the Boundless — {7}, Legendary Creature — Shapeshifter 6/6
// Changeling (This card is every creature type.)
// As Morophon enters, choose a creature type.
// Spells of the chosen type you cast cost {W}{U}{B}{R}{G} less to cast.
//   This effect reduces only the amount of colored mana you pay.
// Other creatures you control of the chosen type get +1/+1.
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
            // "Other creatures you control of the chosen type get +1/+1" (CR 205.3m)
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlOfChosenType,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        // "Spells of the chosen type you cast cost {W}{U}{B}{R}{G} less to cast."
        // Reduces each of the five colors by 1 (CR 601.2f; Morophon ruling 2019-06-14).
        spell_cost_modifiers: vec![SpellCostModifier {
            change: 0,
            filter: SpellCostFilter::HasChosenCreatureSubtype,
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
            colored_mana_reduction: Some(ManaCost {
                white: 1,
                blue: 1,
                black: 1,
                red: 1,
                green: 1,
                ..Default::default()
            }),
        }],
        ..Default::default()
    }
}

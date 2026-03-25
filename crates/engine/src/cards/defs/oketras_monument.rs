// Oketra's Monument — {3}, Legendary Artifact
// White creature spells you cast cost {1} less to cast.
// Whenever you cast a creature spell, create a 1/1 white Warrior creature token with vigilance.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("oketras-monument"),
        name: "Oketra's Monument".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact],
            &[],
        ),
        oracle_text: "White creature spells you cast cost {1} less to cast.\nWhenever you cast a creature spell, create a 1/1 white Warrior creature token with vigilance.".to_string(),
        // CR 601.2f: White creature spells controller casts cost {1} less.
        // Uses ColorAndCreature(White) — compound filter (must be both creature AND white).
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::ColorAndCreature(Color::White),
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
        }],
        abilities: vec![
            // Whenever you cast a creature spell, create a 1/1 white Warrior token with vigilance.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Creature]),
                    noncreature_only: false,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Warrior".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Warrior".to_string())].into_iter().collect(),
                        colors: [Color::White].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Vigilance].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}

// Hermes, Overseer of Elpis — {3}{U}, Legendary Creature — Elder Wizard 2/4
// Whenever you cast a noncreature spell, create a 1/1 blue Bird creature token
//   with flying and vigilance.
// Whenever you attack with one or more Birds, scry 2.
//
// ENGINE-BLOCKED (second ability): "Whenever you attack with one or more Birds" requires
// a once-per-combat attack trigger gated on controlling attackers of a specific subtype.
// WheneverCreatureYouControlAttacks fires once per attacking creature (not once per combat),
// and WheneverYouAttack has no filter. Neither correctly models the batched subtype attack.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hermes-overseer-of-elpis"),
        name: "Hermes, Overseer of Elpis".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elder", "Wizard"],
        ),
        oracle_text: "Whenever you cast a noncreature spell, create a 1/1 blue Bird creature token with flying and vigilance.\nWhenever you attack with one or more Birds, scry 2.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // Whenever you cast a noncreature spell, create a 1/1 blue Bird token with
            // flying and vigilance.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: None,
                    noncreature_only: true,
                    chosen_subtype_filter: false,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Bird".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Bird".to_string())].into_iter().collect(),
                        colors: [Color::Blue].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::Fixed(1),
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Flying, KeywordAbility::Vigilance].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // ENGINE-BLOCKED: "Whenever you attack with one or more Birds, scry 2."
            // No once-per-combat batched subtype attack trigger exists in DSL.
            // WheneverCreatureYouControlAttacks fires per-creature (not per-combat),
            // and WheneverYouAttack has no subtype filter. Omitted per W5 policy.
        ],
        ..Default::default()
    }
}

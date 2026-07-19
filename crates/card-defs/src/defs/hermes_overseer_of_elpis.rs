// Hermes, Overseer of Elpis — {3}{U}, Legendary Creature — Elder Wizard 2/4
// Whenever you cast a noncreature spell, create a 1/1 blue Bird creature token
//   with flying and vigilance.
// Whenever you attack with one or more Birds, scry 2.
//
// PB-OS11 (forced add, self-identified TODO): "Whenever you attack with one or more Birds"
// is a BATCH trigger (CR 508.1m) -- fires ONCE per combat if at least one Bird attacked,
// not once per matching attacker. TriggerCondition::WheneverYouAttack{filter} (PB-OS11)
// expresses this directly via has_subtype: Bird on the declared-attacker set.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hermes-overseer-of-elpis"),
        name: "Hermes, Overseer of Elpis".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            blue: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elder", "Wizard"],
        ),
        oracle_text: "Whenever you cast a noncreature spell, create a 1/1 blue Bird creature \
                      token with flying and vigilance.\nWhenever you attack with one or more \
                      Birds, scry 2."
            .to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // Whenever you cast a noncreature spell, create a 1/1 blue Bird token with
            // flying and vigilance.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: None,
                    noncreature_only: true,
                    chosen_subtype_filter: false,
                    spell_subtype_filter: None,
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
                        supertypes: imbl::OrdSet::new(),
                        keywords: [KeywordAbility::Flying, KeywordAbility::Vigilance]
                            .into_iter()
                            .collect(),
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
            // "Whenever you attack with one or more Birds, scry 2." (CR 508.1m batch trigger.)
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouAttack {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Bird".to_string())),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::Scry {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}

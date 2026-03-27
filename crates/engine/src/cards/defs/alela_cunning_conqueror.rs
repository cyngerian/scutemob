// Alela, Cunning Conqueror — {2UB}, Legendary Creature — Faerie Warlock 2/4
// Flying. Whenever you cast your first spell during each opponent's turn,
// create a 1/1 black Faerie Rogue creature token with flying. Whenever one or
// more Faeries you control deal combat damage to a player, goad target creature
// that player controls.
//
// "First spell per turn" tracking deferred (requires per-turn state counter).
// TODO: Goad effect needs DamagedPlayer resolution for "that player" targeting.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("alela-cunning-conqueror"),
        name: "Alela, Cunning Conqueror".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, black: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Faerie", "Warlock"]),
        oracle_text:
            "Flying\n\
             Whenever you cast your first spell during each opponent's turn, create a 1/1 black Faerie Rogue creature token with flying.\n\
             Whenever one or more Faeries you control deal combat damage to a player, goad target creature that player controls."
                .to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 603.1: fires only during opponent turns (during_opponent_turn: true).
            // "First spell per turn" tracking deferred to later session.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: true,
                    spell_type_filter: None,
                    noncreature_only: false,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Faerie Rogue".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: [Color::Black].into_iter().collect(),
                        supertypes: OrdSet::new(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Faerie".to_string()), SubType("Rogue".to_string())]
                            .into_iter()
                            .collect(),
                        keywords: [KeywordAbility::Flying].into_iter().collect(),
                        count: 1,
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
            },
            // CR 510.3a / CR 603.2c: "Whenever one or more Faeries you control deal combat
            // damage to a player, goad target creature that player controls." — batch trigger
            // with Faerie subtype filter.
            // TODO(PB-37): Goad effect needs DamagedPlayer ForEach support to resolve "that
            // player controls" scoping. DeclaredTarget { index: 0 } with empty targets would
            // panic at resolution, so the effect is a no-op placeholder until PB-37.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Faerie".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::Sequence(vec![]),
                intervening_if: None,
                targets: vec![],
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

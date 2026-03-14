// 55. Alela, Cunning Conqueror — {2UB}, Legendary Creature — Faerie Warlock 2/4;
// Flying. Whenever you cast your first spell during each opponent's turn,
// create a 1/1 black Faerie Rogue creature token with flying. Whenever one or
// more Faeries you control deal combat damage to a player, goad target creature
// that player controls.
//
// M9.4 improvements from M8 simplifications:
// - Trigger 1: WheneverYouCastSpell { during_opponent_turn: true } restricts
// the token trigger to opponent turns only (CR 603.1).
// "First spell per turn" tracking deferred (requires per-turn state counter).
// - Trigger 2: Effect::Goad now implemented; target is the nearest creature
// the damaged player controls. Faerie-filtered trigger remains approximated
// as WhenDealsCombatDamageToPlayer (fires when Alela deals combat damage).
// TODO: Add TriggerCondition::WheneverCreatureTypeYouControlDealsCombatDamage
// with a creature-type filter (Session 1 item 6 plan note).
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
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                intervening_if: None,
            },
            // CR 701.38: Effect::Goad — goad target creature that the damaged player controls.
            // Trigger approximated as WhenDealsCombatDamageToPlayer (fires when Alela
            // itself deals combat damage); Faerie-filtered variant deferred.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::Goad {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                intervening_if: None,
            },
        ],
        color_indicator: None,
        back_face: None,
    }
}

// Sram, Senior Edificer — {1}{W}, Legendary Creature — Dwarf Advisor 2/2
// Whenever you cast an Aura, Equipment, or Vehicle spell, draw a card.
//
// PB-AC7: unblocked by TriggerCondition::WheneverYouCastSpell.spell_subtype_filter
// (Option<Vec<SubType>>, OR-semantics, CR 205.1a).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sram-senior-edificer"),
        name: "Sram, Senior Edificer".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dwarf", "Advisor"],
        ),
        oracle_text: "Whenever you cast an Aura, Equipment, or Vehicle spell, draw a card."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WheneverYouCastSpell {
                during_opponent_turn: false,
                spell_type_filter: None,
                noncreature_only: false,
                chosen_subtype_filter: false,
                spell_subtype_filter: Some(vec![
                    SubType("Aura".to_string()),
                    SubType("Equipment".to_string()),
                    SubType("Vehicle".to_string()),
                ]),
            },
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}

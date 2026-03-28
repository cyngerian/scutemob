// Archmage Emeritus — {2}{U}{U}, Creature — Human Wizard 2/2
// Magecraft — Whenever you cast or copy an instant or sorcery spell, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("archmage-emeritus"),
        name: "Archmage Emeritus".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Magecraft — Whenever you cast or copy an instant or sorcery spell, draw a card.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // Magecraft — instant/sorcery filter applied. "or copy" half is a known gap.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Instant, CardType::Sorcery]),
                    noncreature_only: false,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}

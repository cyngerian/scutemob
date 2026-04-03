// Storm-Kiln Artist — {3}{R}, Creature — Dwarf Shaman 2/2
// Gets +1/+0 for each artifact you control (CDA — TODO).
// Magecraft — Whenever you cast or copy an instant or sorcery spell, create a Treasure token.
//
// TODO: "This creature gets +1/+0 for each artifact you control." — CDA based on artifact count,
// not expressible in DSL (no EffectAmount::CountArtifactsYouControl for P/T modification).
//
// Note: Magecraft triggers on cast OR copy. WheneverYouCastSpell does not cover copies.
// Using approximation (cast only) per W5 — the copy half is a known gap.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("storm-kiln-artist"),
        name: "Storm-Kiln Artist".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: creature_types(&["Dwarf", "Shaman"]),
        oracle_text: "This creature gets +1/+0 for each artifact you control.\nMagecraft — Whenever you cast or copy an instant or sorcery spell, create a Treasure token.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: +1/+0 per artifact CDA — see comment above
            // Magecraft — instant/sorcery filter applied. "or copy" half is a known gap.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Instant, CardType::Sorcery]),
                    noncreature_only: false,
                    chosen_subtype_filter: false,
                },
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
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

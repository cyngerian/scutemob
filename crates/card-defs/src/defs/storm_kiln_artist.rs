// Storm-Kiln Artist — {3}{R}, Creature — Dwarf Shaman 2/2
// Gets +1/+0 for each artifact you control.
// Magecraft — Whenever you cast or copy an instant or sorcery spell, create a Treasure token.
//
// Note: Magecraft triggers on cast OR copy. WheneverYouCastSpell does not cover copies.
// Using approximation (cast only) per W5 — the copy half is a known gap.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("storm-kiln-artist"),
        name: "Storm-Kiln Artist".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Dwarf", "Shaman"]),
        oracle_text: "This creature gets +1/+0 for each artifact you control.\nMagecraft — \
                      Whenever you cast or copy an instant or sorcery spell, create a Treasure \
                      token."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 611.3a, 613.4c: +1/+0 for each artifact you control — static Layer 7c
            // modify on top of the base 2/2 (PB-AC3 CdaModifyPowerToughness, power only).
            AbilityDefinition::CdaModifyPowerToughness {
                power: Some(EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Artifact),
                        ..Default::default()
                    },
                    controller: PlayerTarget::Controller,
                }),
                toughness: None,
            },
            // Magecraft — instant/sorcery filter applied. "or copy" half is a known gap.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Instant, CardType::Sorcery]),
                    noncreature_only: false,
                    chosen_subtype_filter: false,
                    spell_subtype_filter: None,
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
        completeness: Completeness::known_wrong(
            "Magecraft triggers on cast only; the printed 'or copy' half does not trigger",
        ),
        ..Default::default()
    }
}

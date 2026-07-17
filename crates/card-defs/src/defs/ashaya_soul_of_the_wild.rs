// Ashaya, Soul of the Wild — {3}{G}{G}, Legendary Creature — Elemental */*
// Ashaya's power and toughness are each equal to the number of lands you control.
// Nontoken creatures you control are Forest lands in addition to their other types.
//
// CDA (*/*): power: None, toughness: None per KI-4.
// CR 613.4c: PB-AC3 CdaPowerToughness{PermanentCount{Land}} (see Ulvenwald Hydra for the
// pattern) — now authored below.
// TODO: "Nontoken creatures you control are Forest lands in addition to their other
// types" — EffectFilter has no nontoken-exclusion variant (EffectFilter::CreaturesYouControl
// includes tokens). Authoring the type-grant statics as-is would grant Forest/Land types to
// token creatures too (wrong game state per oracle text "Nontoken creatures you control...").
// Omitted until a nontoken-scoped EffectFilter (or equivalent) ships; this is a capability
// gap, not wrong state, since the statics are simply absent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ashaya-soul-of-the-wild"),
        name: "Ashaya, Soul of the Wild".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elemental"],
        ),
        oracle_text: "Ashaya, Soul of the Wild's power and toughness are each equal to the number \
                      of lands you control.\nNontoken creatures you control are Forest lands in \
                      addition to their other types."
            .to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            // CR 613.4c: CDA — power and toughness each equal to the number of lands you
            // control.
            AbilityDefinition::CdaPowerToughness {
                power: EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                    controller: PlayerTarget::Controller,
                },
                toughness: EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                    controller: PlayerTarget::Controller,
                },
            },
            // TODO: "Nontoken creatures you control are Forest lands in addition to their
            // other types" — no nontoken-scoped EffectFilter exists yet. See file-header
            // comment for full disposition.
        ],
        completeness: Completeness::partial(
            "'Nontoken creatures you control are Forest lands in addition to their other types' — \
             EffectFilter has no...",
        ),
        ..Default::default()
    }
}

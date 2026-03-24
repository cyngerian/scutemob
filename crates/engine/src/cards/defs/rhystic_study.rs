// 40. Rhystic Study — {2U}, Enchantment; whenever an opponent casts a spell,
// you may draw a card unless that player pays {1}.
// M9.4: payer is DeclaredTarget { index: 0 } — the opponent who cast the spell.
// The triggering opponent is expected to be passed as target 0 when the
// card-def trigger dispatch system is wired up (currently deferred).
// In the interim the draw always fires (payment never collected) because
// triggered abilities resolve with targets: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rhystic-study"),
        name: "Rhystic Study".to_string(),
        mana_cost: Some(ManaCost { blue: 1, generic: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever an opponent casts a spell, you may draw a card unless that player pays {1}.".to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverOpponentCastsSpell {
                spell_type_filter: None,
                noncreature_only: false,
            },
            effect: Effect::MayPayOrElse {
                cost: Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                // DeclaredTarget { index: 0 } = the specific opponent who cast the spell.
                // This is the correct model (CR 603.1): only "that player" pays, not all
                // opponents. Resolves to an empty list at runtime until trigger context
                // wiring passes the casting opponent as target 0.
                payer: PlayerTarget::DeclaredTarget { index: 0 },
                or_else: Box::new(Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                }),
            },
            intervening_if: None,
            targets: vec![],
        }],
        ..Default::default()
    }
}

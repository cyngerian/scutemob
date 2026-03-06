// Mystic Remora — {U}, Enchantment — Fish; cumulative upkeep {1};
// whenever an opponent casts a noncreature spell, you may draw a card unless that player pays {4}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mystic-remora"),
        name: "Mystic Remora".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Fish"]),
        oracle_text: "Cumulative upkeep {1} (At the beginning of your upkeep, put an age counter on this permanent, then sacrifice it unless you pay its upkeep cost for each age counter on it.)\nWhenever an opponent casts a noncreature spell, you may draw a card unless that player pays {4}.".to_string(),
        abilities: vec![
            // Cumulative upkeep {1} — CR 702.24a
            AbilityDefinition::Keyword(KeywordAbility::CumulativeUpkeep(
                CumulativeUpkeepCost::Mana(ManaCost { generic: 1, ..Default::default() }),
            )),
            AbilityDefinition::CumulativeUpkeep {
                cost: CumulativeUpkeepCost::Mana(ManaCost { generic: 1, ..Default::default() }),
            },
            // TODO: WheneverOpponentCastsSpell does not filter for noncreature spells.
            // A WheneverOpponentCastsNoncreatureSpell trigger condition is needed to express
            // the full oracle text. Using WheneverOpponentCastsSpell as closest approximation;
            // the trigger will fire for creature spells as well until the filter is added.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverOpponentCastsSpell,
                effect: Effect::MayPayOrElse {
                    cost: Cost::Mana(ManaCost { generic: 4, ..Default::default() }),
                    payer: PlayerTarget::DeclaredTarget { index: 0 },
                    or_else: Box::new(Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    }),
                },
                intervening_if: None,
            },
        ],
        ..Default::default()
    }
}

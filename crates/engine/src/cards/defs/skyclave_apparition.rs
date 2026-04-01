// Skyclave Apparition — {1}{W}{W}, Creature — Kor Spirit 2/2
// When this creature enters, exile up to one target nonland, nontoken permanent you
// don't control with mana value 4 or less.
// When this creature leaves the battlefield, the exiled card's owner creates an X/X
// blue Illusion creature token, where X is the mana value of the exiled card.
//
// ETB exile works. LTB token with dynamic X = exiled card's MV is a DSL gap.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skyclave-apparition"),
        name: "Skyclave Apparition".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 2, ..Default::default() }),
        types: creature_types(&["Kor", "Spirit"]),
        oracle_text: "When this creature enters, exile up to one target nonland, nontoken permanent you don't control with mana value 4 or less.\nWhen this creature leaves the battlefield, the exiled card's owner creates an X/X blue Illusion creature token, where X is the mana value of the exiled card.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // ETB: Exile target nonland, nontoken permanent opponent controls with MV ≤ 4.
            // TODO: "nontoken" and "mana value 4 or less" filters not in TargetFilter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::ExileObject {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                intervening_if: None,
                // TODO: "nonland, nontoken, MV ≤ 4" filter — only controller:Opponent available.
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
            // LTB: Exiled card's owner creates X/X blue Illusion token.
            // TODO: Token with dynamic P/T = MV of exiled card. Needs ExiledBySource
            // tracking + ManaValueOf(exiled card) for token P/T. Leaving as Nothing.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenLeavesBattlefield,
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}

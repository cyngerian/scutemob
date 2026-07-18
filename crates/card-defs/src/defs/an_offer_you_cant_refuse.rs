// An Offer You Can't Refuse — {U}, Instant; counter target noncreature spell. Its
// controller creates two Treasure tokens.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("an-offer-you-cant-refuse"),
        name: "An Offer You Can't Refuse".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target noncreature spell. Its controller creates two Treasure \
                      tokens."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // PB-EF2 / An Offer ruling (2022-04-29): the tokens go to the countered spell's
            // controller even if it's an uncounterable spell (a legal but uncounterable
            // target still creates the tokens) — captured by
            // EffectContext::countered_spell_controller in the CounterSpell arm before the
            // cant_be_countered check.
            effect: Effect::Sequence(vec![
                Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    exile_instead: false,
                },
                Effect::CreateToken {
                    spec: TokenSpec {
                        recipient: PlayerTarget::ControllerOfCounteredSpell,
                        ..treasure_token_spec(2)
                    },
                },
            ]),
            targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter {
                non_creature: true,
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

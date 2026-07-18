// Yawgmoth, Thran Physician — {2}{B}{B}, Legendary Creature — Human Cleric 2/4
// Protection from Humans
// Pay 1 life, Sacrifice another creature: Put a -1/-1 counter on up to one target
// creature and draw a card.
// {B}{B}, Discard a card: Proliferate.
//
// PB-EF1 (scutemob-99): "Sacrifice another creature" cost is now expressible via
// Cost::Sacrifice(TargetFilter.exclude_self) → ActivationCost.sacrifice_exclude_self.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("yawgmoth-thran-physician"),
        name: "Yawgmoth, Thran Physician".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Cleric"],
        ),
        oracle_text: "Protection from Humans\nPay 1 life, Sacrifice another creature: Put a -1/-1 \
                      counter on up to one target creature and draw a card.\n{B}{B}, Discard a \
                      card: Proliferate."
            .to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromSubType(SubType("Human".to_string())),
            )),
            // Pay 1 life, Sacrifice another creature: Put a -1/-1 counter on up to one
            // target creature and draw a card.
            // PB-EF1 (CR 109.1): "Sacrifice ANOTHER creature" — Cost::Sacrifice carries
            // TargetFilter.exclude_self, lowered onto ActivationCost.sacrifice_exclude_self
            // and enforced in handle_activate_ability. Cost::PayLife shipped in SR-36
            // (ActivationCost.life_cost). "up to one target" = TargetRequirement::UpToN.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::PayLife(1),
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::You,
                        exclude_self: true,
                        ..Default::default()
                    }),
                ]),
                effect: Effect::Sequence(vec![
                    Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::MinusOneMinusOne,
                        count: 1,
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::UpToN {
                    count: 1,
                    inner: Box::new(TargetRequirement::TargetCreature),
                }],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // {B}{B}, Discard a card: Proliferate.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        black: 2,
                        ..Default::default()
                    }),
                    Cost::DiscardCard,
                ]),
                effect: Effect::Proliferate,
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        // PB-EF1 (scutemob-99): the last blocker — the "another" restriction on the
        // sacrifice cost — is closed. ActivationCost.sacrifice_exclude_self (lowered from
        // Cost::Sacrifice's TargetFilter.exclude_self) is enforced in handle_activate_ability
        // (CR 109.1), so Yawgmoth cannot pay by sacrificing itself. Both activated abilities
        // and Protection from Humans are implemented. Complete.
        ..Default::default()
    }
}

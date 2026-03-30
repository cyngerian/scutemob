// Eiganjo, Seat of the Empire — Legendary Land, {T}: Add {W}. Channel ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eiganjo-seat-of-the-empire"),
        name: "Eiganjo, Seat of the Empire".to_string(),
        mana_cost: None,
        types: full_types(&[SuperType::Legendary], &[CardType::Land], &[]),
        oracle_text: "{T}: Add {W}.\nChannel — {2}{W}, Discard this card: It deals 4 damage to target attacking or blocking creature. This ability costs {1} less to activate for each legendary creature you control.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // Channel — {2}{W}, Discard this card: 4 damage to target creature.
            // TODO: Target filter should restrict to "attacking or blocking creature" —
            //   requires TargetFilter.is_attacking/is_blocking flags (combat state query).
            //   DSL gap: TargetFilter has no field to constrain to attacking/blocking status.
            //   Using TargetCreature as approximation (overly broad — can target non-combatants).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, white: 1, ..Default::default() }),
                    Cost::DiscardSelf,
                ]),
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(4),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        // CR 602.2b + 601.2f: Channel ability (index 0) costs {1} less per legendary creature.
        activated_ability_cost_reductions: vec![(
            0,
            SelfActivatedCostReduction::PerPermanent {
                per: 1,
                filter: TargetFilter {
                    legendary: true,
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                },
                controller: PlayerTarget::Controller,
            },
        )],
        ..Default::default()
    }
}

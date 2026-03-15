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
            },
            // Channel — {2}{W}, Discard this card: 4 damage to target creature.
            // TODO: Target filter should restrict to "attacking or blocking creature".
            // TODO: Cost reduction — {1} less per legendary creature you control.
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
            },
        ],
        ..Default::default()
    }
}

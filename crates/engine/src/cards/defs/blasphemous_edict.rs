// Blasphemous Edict — {3}{B}{B}, Sorcery
// This spell costs {B}{B} less to cast if there are thirteen or more creatures
// on the battlefield.
// Each player sacrifices a creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blasphemous-edict"),
        name: "Blasphemous Edict".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "This spell costs {B}{B} less to cast if there are thirteen or more creatures on the battlefield.\nEach player sacrifices a creature.".to_string(),
        abilities: vec![
            // TODO: Conditional cost reduction "{B}{B} less if 13+ creatures on the battlefield"
            // is not expressible in the current DSL. Blocked on a
            // Condition::CreaturesOnBattlefieldAtLeast(N) primitive. (PB-SFT scope boundary.)
            AbilityDefinition::Spell {
                // PB-SFT (CR 701.21a + CR 109.1): creature-only filter applied.
                // Each player sacrifices a creature of their choice.
                effect: Effect::SacrificePermanents {
                    player: PlayerTarget::EachPlayer,
                    count: EffectAmount::Fixed(1),
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

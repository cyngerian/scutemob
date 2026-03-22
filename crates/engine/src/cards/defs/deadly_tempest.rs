// Deadly Tempest — {4}{B}{B} Sorcery
// Destroy all creatures. Each player loses life equal to the number of
// creatures they controlled that were destroyed this way.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("deadly-tempest"),
        name: "Deadly Tempest".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Destroy all creatures. Each player loses life equal to the number of creatures they controlled that were destroyed this way.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 701.8: Destroy all creatures.
            // TODO: The "each player loses life equal to creatures they controlled" requires
            // per-player tracking of how many creatures each player lost in the DestroyAll.
            // The current DSL only supports LastEffectCount (total count) via
            // EffectAmount::LastEffectCount, not per-player breakdowns. When per-player
            // destroy-count tracking is added to EffectContext, replace this with:
            //   ForEach { over: EachPlayer, effect: LoseLife { amount: DestroyedCountFor(player) } }
            effect: Effect::DestroyAll {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                },
                cant_be_regenerated: false,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

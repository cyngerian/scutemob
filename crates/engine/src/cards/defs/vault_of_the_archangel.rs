// Vault of the Archangel — Land, {T}: Add {C}. {2}{W}{B}, {T}: grant deathtouch+lifelink until EOT (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vault-of-the-archangel"),
        name: "Vault of the Archangel".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{2}{W}{B}, {T}: Creatures you control gain deathtouch and lifelink until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: {2}{W}{B}, {T}: Creatures you control gain deathtouch and lifelink until end of turn.
            // DSL gap: no ApplyContinuousEffect with AllCreaturesYouControl + EffectDuration::EndOfTurn
            // and multi-keyword grant in one activation cost.
        ],
        ..Default::default()
    }
}

// Bleachbone Verge — Land, {T}: Add {B}. {T}: Add {W} (only if you control Plains or Swamp).
// TODO: conditional {T}: Add {W} requires Condition::ControlsPermanentWithSubtype — not yet in DSL
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bleachbone-verge"),
        name: "Bleachbone Verge".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {B}.\n{T}: Add {W}. Activate only if you control a Plains or a Swamp.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: {T}: Add {W} — requires conditional activation (control Plains or Swamp)
        ],
        ..Default::default()
    }
}

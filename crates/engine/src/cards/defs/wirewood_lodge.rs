// Wirewood Lodge — Land
// {T}: Add {C}. {G},{T}: Untap target Elf (untap ability not expressible).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wirewood-lodge"),
        name: "Wirewood Lodge".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{G}, {T}: Untap target Elf.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // TODO: {G},{T}: Untap target Elf — tap-plus-pay-mana cost with
            // untap-creature effect targeting a specific subtype is not
            // expressible in the DSL (no Effect::UntapPermanent or combined
            // mana+tap Cost variant; no subtype target filter for untap)
        ],
        ..Default::default()
    }
}

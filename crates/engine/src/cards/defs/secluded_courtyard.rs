// Secluded Courtyard — Land, ETB choose creature type, {T}: Add {C}; restricted any-color mana
// TODO: "As this land enters, choose a creature type" — ETB choice not expressible in DSL
// TODO: {T}: Add one mana of any color restricted to chosen creature type spells/abilities
// — spending restriction on mana not expressible in DSL; implementing as unrestricted
// AddManaAnyColor would produce wrong behavior (permits spending on any spell), so left as TODO
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("secluded-courtyard"),
        name: "Secluded Courtyard".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "As this land enters, choose a creature type.\n{T}: Add {C}.\n{T}: Add one mana of any color. Spend this mana only to cast a creature spell of the chosen type or activate an ability of a creature source of the chosen type.".to_string(),
        abilities: vec![
            // {T}: Add {C}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}

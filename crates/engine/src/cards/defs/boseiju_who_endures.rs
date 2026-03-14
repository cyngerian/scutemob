// Boseiju, Who Endures — Legendary Land
// {T}: Add {G}. Channel ability (discard + mana cost) not expressible in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boseiju-who-endures"),
        name: "Boseiju, Who Endures".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {G}.\nChannel — {1}{G}, Discard this card: Destroy target artifact, enchantment, or nonbasic land an opponent controls. That player may search their library for a land card with a basic land type, put it onto the battlefield, then shuffle. This ability costs {1} less to activate for each legendary creature you control.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: Channel ability — discard-as-cost activated ability with
            // variable mana cost (scaling down per legendary creature) and
            // multi-type target filter (artifact, enchantment, or nonbasic land)
            // are not expressible in the DSL
        ],
        ..Default::default()
    }
}

// Eternal Witness — {1}{G}{G}, Creature — Human Shaman 2/1.
// "When Eternal Witness enters the battlefield, you may return target card from
// your graveyard to your hand."
// CR 603.2: ETB triggered ability — return a card from graveyard to hand.
// Simplification: draws one card (approximation; actual effect returns a specific
// graveyard card, but the engine lacks a "return from graveyard to hand" targeted
// trigger effect — modeled as DrawCards as closest functional approximation).
// TODO: replace with a proper ReturnFromGraveyardToHand effect when implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eternal-witness"),
        name: "Eternal Witness".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: creature_types(&["Human", "Shaman"]),
        oracle_text: "When Eternal Witness enters the battlefield, you may return target card from your graveyard to your hand.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            // ETB: return a card from graveyard to hand.
            // Approximation: DrawCards(1) — replace with ReturnFromGraveyardToHand when available.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
            },
        ],
    }
}

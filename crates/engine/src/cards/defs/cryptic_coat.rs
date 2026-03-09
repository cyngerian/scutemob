// Cryptic Coat — {2}{U}, Artifact — Equipment
// When this Equipment enters, cloak the top card of your library, then attach this
// Equipment to it. (To cloak a card, put it onto the battlefield face down as a 2/2
// creature with ward {2}. Turn it face up any time for its mana cost if it's a
// creature card.)
// Equipped creature gets +1/+0 and can't be blocked.
// {1}{U}: Return this Equipment to its owner's hand.
//
// DSL gaps:
// - "then attach this Equipment to it" — no AttachToNewlyCreated effect primitive. TODO.
// - Static grant (+1/+0, can't be blocked) — no EquippedCreatureGrant continuous effect. TODO.
// - {1}{U}: Return to hand — no ReturnToHand activated ability. TODO.
// Core Cloak mechanic (ETB Cloak) is fully represented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cryptic-coat"),
        name: "Cryptic Coat".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "When this Equipment enters, cloak the top card of your library, \
then attach this Equipment to it. (To cloak a card, put it onto the battlefield face \
down as a 2/2 creature with ward {2}. Turn it face up any time for its mana cost if \
it's a creature card.)\nEquipped creature gets +1/+0 and can't be blocked.\n\
{1}{U}: Return this Equipment to its owner's hand."
            .to_string(),
        abilities: vec![
            // CR 701.58a: ETB trigger — cloak the top card of the controller's library.
            // TODO: "then attach this Equipment to it" has no DSL primitive. Deferred.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Cloak { player: PlayerTarget::Controller },
                intervening_if: None,
            },
            // TODO: Static grant — equipped creature gets +1/+0 and can't be blocked.
            // No EquippedCreatureGrant continuous effect primitive. Deferred.

            // TODO: {1}{U}: Return this Equipment to its owner's hand.
            // No ReturnToHand activated ability. Deferred.
        ],
        ..Default::default()
    }
}

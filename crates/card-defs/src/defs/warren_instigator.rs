// Warren Instigator — {R}{R}, Creature — Goblin Berserker 1/1
// Double strike
// Whenever this creature deals damage to an opponent, you may put a Goblin
// creature card from your hand onto the battlefield.
//
// PB-DX36 (`OOS-CARDS2-6`): the trigger CONDITION is now expressible —
// TriggerCondition::WhenDealsDamage { recipient: DamageRecipient::Opponent }
// (CR 603.2) closes the "deals damage to an opponent" gap this def used to
// carry. It is deliberately NOT declared while the effect is unimplementable —
// see the comment above `abilities`. Two blockers survive:
// (a) no effect puts a FILTERED (Goblin creature) card from hand onto the
// battlefield — Effect::PutLandFromHandOntoBattlefield is land-only; (b) the
// costless "you may" is inexpressible (see goblin_lackey/curiosity/ophidian_eye
// for the same gap).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("warren-instigator"),
        name: "Warren Instigator".to_string(),
        mana_cost: Some(ManaCost {
            red: 2,
            ..Default::default()
        }),
        types: creature_types(&["Goblin", "Berserker"]),
        oracle_text: "Double strike\nWhenever this creature deals damage to an opponent, you may \
                      put a Goblin creature card from your hand onto the battlefield."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        // NOT authored as a `Triggered` ability with an `Effect::Nothing` body, and the
        // reason is this batch's own subject matter: `OOS-CARDS2-6` is a trigger that
        // exists and does nothing. Declaring the now-expressible condition here would put
        // a real, respondable, no-op ability on the stack (CR 603.2, CR 113.7a) — a claim
        // the effect half cannot honour — where today the def makes no claim at all.
        // `goblin_lackey` carries that shape already and is repaired in place rather than
        // copied; a NEW one is not created. Re-author both the day blocker (a) closes.
        // TODO: "you may put a Goblin creature card from your hand onto the battlefield"
        // — two gaps, and the TRIGGER is no longer one of them (PB-DX36 shipped
        // `TriggerCondition::WhenDealsDamage { recipient: DamageRecipient::Opponent }`).
        // (a) no effect puts a FILTERED card from hand onto the battlefield —
        // `Effect::PutLandFromHandOntoBattlefield` is land-only; (b) the costless "you
        // may" is inexpressible (`OOS-DX35-5`).
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::DoubleStrike)],
        completeness: Completeness::partial(
            "Blocked: (a) no effect puts a filtered (Goblin creature) card from hand onto the \
             battlefield — Effect::PutLandFromHandOntoBattlefield is land-only; (b) 'you may' is \
             inexpressible (Effect::Choose always takes the first option, effects/mod.rs:3190). \
             Trigger currently resolves to Effect::Nothing.",
        ),
        ..Default::default()
    }
}

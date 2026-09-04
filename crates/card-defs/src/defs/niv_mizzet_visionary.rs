// Niv-Mizzet, Visionary — {4}{U}{R}, Legendary Creature — Dragon Wizard 5/5
// Flying
// You have no maximum hand size.
// Whenever a source you control deals noncombat damage to an opponent, you draw that many cards.
//
// Flying is implemented.
//
// "You have no maximum hand size" now expressed via KeywordAbility::NoMaxHandSize (PB-AC8).
//
// TODO: ENGINE-BLOCKED — "whenever a source you control deals noncombat damage to an
// opponent, draw that many cards". NARROWED by PB-DX36 (`OOS-CARDS2-6`): this note used to
// name TWO gaps and one of them is now closed. The variable amount IS expressible —
// `EffectAmount::DamageDealt` (CR 608.2h/113.7a) reads the triggering damage event's amount,
// which is exactly "that many". What survives is the TRIGGER CONDITION, and PB-DX36's
// `WhenDealsDamage` does not reach it on two axes: its subject is THIS permanent, not "a
// source YOU CONTROL", and it fires on any damage where this card wants NONcombat damage
// only. Filed as `OOS-DX36-3`.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("niv-mizzet-visionary"),
        name: "Niv-Mizzet, Visionary".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            blue: 1,
            red: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Wizard"],
        ),
        oracle_text: "Flying\nYou have no maximum hand size.\nWhenever a source you control deals \
                      noncombat damage to an opponent, you draw that many cards."
            .to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::NoMaxHandSize),
            // TODO: ENGINE-BLOCKED — any-source, controller-scoped, NONcombat-only damage
            // trigger (see header; the variable-amount half closed in PB-DX36).
        ],
        completeness: Completeness::partial(
            "ENGINE-BLOCKED — 'whenever a source you control deals noncombat damage to an \
             opponent, draw that many cards'. NARROWED by PB-DX36: the 'that many' half is now \
             expressible (EffectAmount::DamageDealt). The surviving blocker is the trigger \
             condition — any SOURCE YOU CONTROL (not this permanent) and NONcombat damage only; \
             TriggerCondition::WhenDealsDamage is self-scoped and damage-kind agnostic. \
             OOS-DX36-3.",
        ),
        ..Default::default()
    }
}

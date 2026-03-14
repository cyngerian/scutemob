// 52. Rest in Peace — {1W}, Enchantment.
// "When Rest in Peace enters the battlefield, exile all cards from all
// graveyards. If a card would be put into a graveyard from anywhere,
// exile it instead."
//
// ETB trigger: queued via queue_carddef_etb_triggers as PendingTrigger, placed on
// the stack at the next priority window (CR 603.3). Opponents may respond before
// the exile effect resolves.
// Ongoing replacement: registered via register_permanent_replacement_abilities.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rest-in-peace"),
        name: "Rest in Peace".to_string(),
        mana_cost: Some(ManaCost { white: 1, generic: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text:
            "When Rest in Peace enters the battlefield, exile all cards from all graveyards.\n\
             If a card would be put into a graveyard from anywhere, exile it instead."
                .to_string(),
        abilities: vec![
            // CR 603.3, 603.6a: ETB triggered ability — exile all cards from all graveyards.
            // Queued as PendingTrigger by queue_carddef_etb_triggers; resolves via stack.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::ForEach {
                    over: ForEachTarget::EachCardInAllGraveyards,
                    effect: Box::new(Effect::ExileObject {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    }),
                },
                intervening_if: None,
            },
            // CR 614.1a: Replacement — any card going to any graveyard → exile instead.
            // is_self: false — global effect, not tied to Rest in Peace itself.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldChangeZone {
                    from: None,
                    to: ZoneType::Graveyard,
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}

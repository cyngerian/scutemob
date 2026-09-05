// 112. Windbrisk Heights — Land; Hideaway 4; enters tapped; {T}: {W}; {W},{T}: play exiled card.
// CR 702.75: Hideaway 4 triggers on ETB: look at top 4, exile one face-down, put rest on bottom.
// CR 702.75b: older Hideaway cards errata'd to "Hideaway 4" + separate "enters tapped" line.
// The play condition ("attacked with 3+ creatures this turn") is
// Condition::YouAttackedWithNOrMoreCreaturesThisTurn(3), reading
// PlayerState.creatures_declared_as_attackers_this_turn -- a per-turn, deduplicated SET,
// not a per-declaration count.
//
// CLOSED by PB-DX53 (`OOS-DX21-1`). History: CARDS-2 review (scutemob-181) first authored
// this condition against `Condition::YouAttackedWithNOrMore`, which was per-DECLARATION
// (the field was ASSIGNED, not accumulated, at each `DeclareAttackers`). PB-DX21
// (scutemob-200, `OOS-M11-9`) closed the WITHIN-one-combat half of the resulting defect: the
// engine now rejects a second `DeclareAttackers` in the same combat
// (`GameStateError::AlreadyDeclaredAttackers`). The EXTRA-COMBAT half survived that fix,
// because a fresh `CombatState` is installed at each `BeginningOfCombat` (CR 500.8 adds the
// phase; CR 506.1 gives every combat phase its own declare-attackers step -- NOT CR 506.5,
// which defines "attacks alone" and was the first draft's cite):
// attacking with three in combat 1 and then one in combat 2 dropped the count to one and this
// land went dead for the rest of the turn, which the printed card does not do (ruling
// 2007-10-01: "at any point in the turn"). It was also not deduplicated by creature.
//
// PB-DX53 closed it structurally rather than patching the shared field: the one Condition
// variant that both this card and `legions_landing.rs` read could not carry both cards' CR
// concepts (CR 508.3d per-declaration vs. this ruling's per-turn, deduplicated scope), so the
// DSL split into two variants over two PlayerState fields. This card now reads
// `Condition::YouAttackedWithNOrMoreCreaturesThisTurn(3)` against
// `creatures_declared_as_attackers_this_turn: OrdSet<ObjectId>` -- accumulated across every
// combat phase on an extra-combat turn, deduplicated by ObjectId (CR 400.7 identity, matching
// the ruling's own "counts only once" for a creature declared as an attacker in two different
// attack phases), and never populated by a CR 508.4 entrant (put onto the battlefield
// attacking), which the ruling's third sentence excludes ("you never attacked with it") and
// which the engine excludes by construction: the write site reads the DECLARATION command's
// own attacker list, never the combat-wide `combat.attackers` map that entrants also occupy.
//
// `legions_landing.rs`'s "Whenever you attack with three or more creatures" trigger is a
// SEPARATE class (CR 508.3d) and was never a member of this residual -- it fires once per
// declaration and its count gate correctly reads the SAME declaration that fired it via the
// sibling `Condition::YouAttackedWithNOrMoreThisDeclaration`, so attacking with 1 creature in
// a later combat correctly does not (re-)satisfy it. That card's behaviour is unchanged by
// this fix, by construction. CR 508.6 ("has attacked [a player]") is a boolean predicate with
// no count or turn-scope content; it does not warrant either card's behaviour and must not be
// cited for this class.
//
// This condition is CR-faithful to the printed card and the 2007-10-01 ruling as of PB-DX53 --
// per-turn, deduplicated, CR 508.4-entrant-excluded. It remains a strict improvement on the
// `None` an even earlier draft had (which let the exiled card be played with no attack at all).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("windbrisk-heights"),
        name: "Windbrisk Heights".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "Hideaway 4 (When this land enters, look at the top four cards of your \
                      library, exile one face down, then put the rest on the bottom in a random \
                      order.)\nThis land enters tapped.\n{T}: Add {W}.\n{W}, {T}: You may play \
                      the exiled card without paying its mana cost if you attacked with three or \
                      more creatures this turn."
            .to_string(),
        abilities: vec![
            // CR 702.75: Hideaway 4 — ETB trigger wired via KeywordAbility::Hideaway(4).
            AbilityDefinition::Keyword(KeywordAbility::Hideaway(4)),
            // CR 614.1c: self-replacement — this land enters the battlefield tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // {T}: Add {W} (no Plains subtype on the printed card; ability is explicit).
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // {W}, {T}: Play the exiled card without paying its mana cost, if you attacked
            // with three or more creatures this turn (CR ruling 2007-10-01: any point in the
            // turn, counted by distinct creatures declared as attackers).
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        white: 1,
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::PlayExiledCard,
                timing_restriction: None,
                targets: vec![],
                activation_condition: Some(Condition::YouAttackedWithNOrMoreCreaturesThisTurn(3)),
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        ..Default::default()
    }
}

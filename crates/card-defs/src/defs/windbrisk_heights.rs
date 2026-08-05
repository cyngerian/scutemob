// 112. Windbrisk Heights — Land; Hideaway 4; enters tapped; {T}: {W}; {W},{T}: play exiled card.
// CR 702.75: Hideaway 4 triggers on ETB: look at top 4, exile one face-down, put rest on bottom.
// CR 702.75b: older Hideaway cards errata'd to "Hideaway 4" + separate "enters tapped" line.
// The play condition ("attacked with 3+ creatures this turn") is
// Condition::YouAttackedWithNOrMore(3), reading PlayerState.attackers_declared_this_turn.
//
// KNOWN RESIDUAL, stated rather than claimed away (CARDS-2 review, scutemob-181; narrowed by
// PB-DX21, scutemob-200): that field is ASSIGNED, not accumulated -- `rules/combat.rs` sets it
// to the size of the latest declaration and says so in its own comment.
//
// PB-DX21 (CR 508.1, `OOS-M11-9`) closed the WITHIN-one-combat half of this: the engine now
// rejects a second `DeclareAttackers` in the same combat (`GameStateError::
// AlreadyDeclaredAttackers`), so `attackers_declared_this_turn` can no longer be
// re-assigned mid-combat by a repeated declaration of the SAME combat.
//
// The EXTRA-COMBAT half survives, unaffected by that fix, because the guard is scoped to one
// combat phase by design (CR 500.8/506.5 -- a fresh `CombatState` is installed at each
// `BeginningOfCombat`). On a turn with an extra combat (`Effect::AdditionalCombatPhase` is
// implemented), attacking with three in combat 1 and then one in combat 2 still drops the
// count to one and this land still goes dead for the rest of the turn, which the printed card
// does not do (ruling 2007-10-01: "at any point in the turn"). It is also still not
// deduplicated by creature. Filed as `OOS-DX21-1`; closing it needs the field to become a
// per-turn accumulation with per-creature dedup, which is a different primitive from
// PB-DX21's once-per-combat guard.
//
// `OOS-DX21-1` is SCOPED TO THIS CARD ALONE (PB-DX21 review, finding M3) -- do NOT migrate
// `legions_landing.rs`'s "Whenever you attack with three or more creatures" trigger into this
// class. That trigger is CR 508.3d's per-DECLARATION family: it fires once per declaration and
// its count gate correctly reads the SAME declaration that fired it, so attacking with 1
// creature in a later combat correctly does not (re-)satisfy it -- that is the card working
// as printed, not a defect. This card's activation condition is the genuinely turn-scoped one
// (ruling 2007-10-01, "at any point in the turn"), which is why it and only it is the residual.
// CR 508.6 ("has attacked [a player]") is a boolean predicate with no count or turn-scope
// content; it does not warrant either card's behaviour and must not be cited for this class.
//
// An earlier draft of this comment cited the 2007-10-01 ruling as though the primitive
// implemented it; it does not, and asserting fidelity a primitive does not have is exactly
// what `braided_net.rs` was demoted for. The condition is still a strict improvement on the
// `None` it replaced (which let the exiled card be played with no attack at all).
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
                activation_condition: Some(Condition::YouAttackedWithNOrMore(3)),
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        ..Default::default()
    }
}

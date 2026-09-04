//! Read-only advisory query surface for UI/simulator callers (M11-local Session 3 §B).
//!
//! Everything here is a **query**, never an action: no function in this module mutates
//! `GameState`, issues an `Event`, or has any side effect. `process_command` (via
//! `crate::rules::engine`) remains the sole authority for changing game state
//! (Architecture Invariants 1 and 3) — a browser client or bot calls the functions here
//! to populate a UI (e.g. "here are your legal targets"), then still submits a `Command`
//! through the normal path, which independently re-validates everything. A value returned
//! from here can still be rejected at cast/activation time.
//!
//! Every function delegates to the same target-requirement lookups and target-legality
//! checkers `rules::casting` uses for `handle_cast_spell` / `handle_activate_ability`
//! (`card_def_target_requirements`, `spell_mode_selection`, `per_mode_target_requirements`,
//! `validate_targets_inner`) rather than re-deriving any rule — re-deriving target legality
//! outside the engine is exactly the drift class OOS-RS-2 was (see the M11-local session
//! plan §1 fact 4).
use crate::cards::TargetRequirement;
use crate::rules::casting;
use crate::rules::combat;
use crate::rules::layers::calculate_characteristics;
use crate::state::{
    AltCostKind, AttackTarget, CardType, EnchantTarget, GameState, KeywordAbility, ManaCost,
    ObjectId, PlayerId, SubType, Target, ZoneId,
};

/// CR 601.2c — the target requirements a spell cast from `card` announces, honouring
/// Aftermath (CR 702.127a), Overload (CR 702.96b -> empty) and per-mode requirements
/// (CR 700.2c/700.2f). Shares `casting.rs`'s helpers so the two cannot drift.
///
/// **Signature deviation from the M11-local session plan's §3 sketch**: this takes an
/// `alt_cost: Option<AltCostKind>` parameter the plan's sketch omits. It is required —
/// `casting_with_overload` (`casting.rs:1163`) and `casting_with_aftermath`
/// (`casting.rs:533`) are both *caster-intent* flags derived from the `CastSpell`
/// command's `alt_cost`, not from state alone, so without this parameter the Overload and
/// Aftermath rules are unreachable here and the CR 702.96b test is unwritable.
/// `AltCostKind` is already a public type (`crate::state::AltCostKind`, re-exported from
/// `mtg-card-types`), so this does not introduce a new public type and cannot move the
/// wire fingerprint (`PROTOCOL_SCHEMA_FINGERPRINT` closes over `Command`/`GameEvent`/
/// `Effect`/`Characteristics`, not over free-function query parameters).
///
/// Missing object, missing `card_id`, or an unregistered card all yield `vec![]` — this
/// function never panics and never unwraps.
///
/// **Two deliberate divergences from `handle_cast_spell`, both narrower than the
/// "cannot drift" guarantee the shared helpers give the card-def lookup itself:**
///
/// 1. An empty `modes_chosen` on a spell with `ModeSelection.mode_targets: Some(_)`
///    yields `vec![]` here, where `handle_cast_spell` would reach its `vec![0]`
///    fail-safe (`casting.rs`, the `indices` derivation). Reporting "no targets until
///    you announce your modes" is the CR 601.2b-faithful answer for a *query* — post
///    PB-DP3 the engine rejects an unannounced modal cast anyway, so the fail-safe is
///    unreachable in practice and mirroring it here would advertise targets for a mode
///    the caster has not chosen.
/// 2. The Aftermath keyword is read from **layer-resolved** characteristics here,
///    where `casting.rs:533-538` reads the raw `card_obj.characteristics.keywords`.
///    The layer-resolved read is the more CR 613.1f-correct one and the two agree for
///    every shipped Aftermath card (they are cast from a graveyard, where continuous
///    effects granting or removing Aftermath do not arise). Left asymmetric rather
///    than "fixed" in either direction, because changing the cast path is a behaviour
///    change and this batch's `casting.rs` edits are refactor-only.
/// 3. (PB-DX20 §4.5) Bestow (CR 702.103b) is applied here to a LOCAL CLONE of `chars`,
///    mirroring the transform `casting.rs`'s Step 1b applies to its own (mutable) `chars`
///    at `:980-988`, long before that function's synthesis point. Both callers already
///    compute layer-resolved characteristics for their own reasons, so this is a
///    query-side RE-DERIVATION of caster intent — the same shape divergences 1 and 2
///    above already are — not a new layer walk. `casting::get_bestow_cost` is the exact
///    eligibility check `casting.rs:975` uses. Bestow is not reachable from the browser
///    today (`StubProvider` enumerates no alt-cost casts, CLAUDE.md M11-local R4, and
///    every `spell_target_requirements` caller here passes `alt_cost: None`); the value
///    is that the cast path and this query cannot drift the day that changes.
///
/// `fuse` (PB-DX44, `OOS-DX29-12`, CR 702.102d): mirrors `casting.rs`'s own
/// `casting_with_fuse` caster-intent flag — `true` when the caller's prospective
/// `CastSpell` is going to announce `AdditionalCost::Fuse`. Like `alt_cost`, this cannot
/// be derived from state alone; it names an intent the `Command` has not been built yet.
/// A `true` here appends the fused right half's own targets after the left half's,
/// through the SAME `card_def_target_requirements` call `handle_cast_spell` makes, so the
/// offer and the cast cannot disagree about the count. Every pre-existing caller in this
/// tree passes `false` for the same reason they pass `modes_chosen: &[]` — the picker
/// renders before the human has chosen whether to fuse, exactly as it renders before the
/// human has chosen modes (divergence 1's own doc).
///
/// `alt_cost: Some(AltCostKind::SplitRightHalf)` (PB-DX44, `OOS-DX29-9`, CR 709.4): unlike
/// `fuse`, casting ONLY the right half is expressed through the existing `alt_cost`
/// parameter, not a new bool — `AltCostKind` is already this function's "which
/// face/half/mode" channel (see `casting_with_aftermath`, `casting_with_bestow`). A `true`
/// derivation here REPLACES the returned requirements with the right half's own targets
/// alone (never the left half's, never both), through the same shared
/// `card_def_target_requirements` call, gated on the same `get_fuse_data` existence check
/// `casting.rs` uses so the offer and the cast agree about which defs even have a right
/// half.
pub fn spell_target_requirements(
    state: &GameState,
    card: ObjectId,
    modes_chosen: &[usize],
    alt_cost: Option<AltCostKind>,
    fuse: bool,
) -> Vec<TargetRequirement> {
    let Some(obj) = state.objects().get(&card) else {
        return vec![];
    };
    let Some(chars) = calculate_characteristics(state, card) else {
        return vec![];
    };
    let card_id = obj.card_id.as_ref();

    // CR 702.96a/b: Overload is an alternative cost that replaces "target" with "each" —
    // an overloaded spell has no targets. Eligibility is the *same call* `casting.rs:1203`
    // makes to establish `casting_with_overload` — `get_overload_cost(...).is_some()`, i.e.
    // the card def carries an `AbilityDefinition::Overload { cost }` — not a parallel
    // re-derivation from the `KeywordAbility::Overload` marker. Delegating keeps this
    // query and the cast path from drifting, which is the whole point of §A's shared
    // helpers, and it leaves `KeywordAbility::Overload` a pure marker under SR-5.
    let casting_with_overload = alt_cost == Some(AltCostKind::Overload)
        && casting::get_overload_cost(&obj.card_id, &state.card_registry).is_some();
    if casting_with_overload {
        return vec![];
    }

    // CR 702.127a: Aftermath — same three conjuncts as `casting.rs:533-538`.
    let casting_with_aftermath = alt_cost == Some(AltCostKind::Aftermath)
        && matches!(obj.zone, ZoneId::Graveyard(_))
        && chars.keywords.contains(&KeywordAbility::Aftermath);

    // CR 702.102a (PB-DX44): mirrors `casting.rs:1279`'s own gate — a fused cast
    // requires the Fuse KEYWORD, not merely the `AbilityDefinition::Fuse` data carrier
    // (`pb_dx29_cost_kind_surface.rs`'s `p2a` covers a corpus def with the data carrier
    // and no marker). Fuse and Aftermath never combine (`casting.rs`'s own mutual
    // exclusion, Step 1h), so the aftermath flag wins if somehow both were asked for.
    // Computed here, BEFORE `chars` is moved into `eff_chars` below — Bestow never
    // touches the Fuse keyword, so reading it from `chars` rather than `eff_chars` is
    // not a divergence.
    let casting_with_fuse =
        fuse && !casting_with_aftermath && chars.keywords.contains(&KeywordAbility::Fuse);

    // CR 709.4 (PB-DX44, `OOS-DX29-9`): mirrors `casting.rs`'s own `cast_right_half`
    // caster-intent derivation (`alt_cost == Some(AltCostKind::SplitRightHalf)`), gated
    // on the SAME `get_fuse_data` lookup the cast path uses -- so the offer layer and the
    // cast agree about which defs have a DSL right half at all, without a second
    // implementation of "does this card have a `AbilityDefinition::Fuse`". Unlike
    // `casting_with_fuse`, this does NOT gate on the `Fuse` KEYWORD (CR 709.4's
    // single-half cast is legal on any split card, not only fusable ones -- see
    // `casting::card_def_target_requirements`'s own doc). Mutually exclusive with both
    // `casting_with_aftermath` and `casting_with_fuse` by construction (`alt_cost` can
    // only be one variant at a time, and `fuse` + `alt_cost: Some(SplitRightHalf)` is
    // rejected at cast time), so no precedence ordering is needed here.
    let casting_right_half = alt_cost == Some(AltCostKind::SplitRightHalf)
        && casting::get_fuse_data(&obj.card_id, &state.card_registry).is_some();

    // CR 702.103b (PB-DX20 §4.5, divergence 3 above): if cast bestowed, apply the SAME
    // keyword transform `casting.rs:980-988` applies to its own `chars`, to a LOCAL
    // CLONE — `chars` itself must stay untransformed for every other caller.
    let casting_with_bestow = alt_cost == Some(AltCostKind::Bestow)
        && casting::get_bestow_cost(&obj.card_id, &state.card_registry).is_some();
    let eff_chars = if casting_with_bestow {
        let mut c = chars.clone();
        c.card_types.remove(&CardType::Creature);
        c.card_types.insert(CardType::Enchantment);
        c.subtypes.insert(SubType("Aura".to_string()));
        c.keywords
            .insert(KeywordAbility::Enchant(EnchantTarget::Creature));
        c
    } else {
        chars
    };

    let (requirements, _cant_be_countered) = casting::card_def_target_requirements(
        state,
        card_id,
        casting_with_aftermath,
        casting_with_fuse,
        casting_right_half,
    );

    // CR 702.127a: aftermath suppresses per-mode targets (mirrors `casting.rs:3689`'s
    // `if casting_with_aftermath { None } else { ... }`). CR 303.4a (PB-DX20 §5 Step 4c):
    // wrap in the shared Aura synthesis so this return point cannot drift from the cast
    // path either.
    if casting_with_aftermath {
        return casting::aura_spell_target_requirements(&eff_chars, requirements);
    }

    // CR 700.2c/700.2f: if the spell has per-mode target requirements, they replace the
    // flat `Spell.targets` list for the chosen modes. CR 303.4a: wrap the final result in
    // the shared Aura synthesis (§5 Step 4c) — an Aura never has per-mode targets today
    // (no shipped card combines the two), so this is a no-op guard in practice, not a
    // behaviour change.
    //
    // `/review` finding 5 (PB-DX44): mirror `casting.rs`'s SAME short-circuit — a
    // right-half-only cast (CR 709.4) must never let a modal LEFT half's per-mode
    // targets replace the right half's own requirements. No shipped card combines
    // the Fuse-right-half DSL carrier with per-mode targets today, so
    // `casting::spell_mode_selection` already returns `None` for every corpus
    // right-half member and this is a latent-gap closure, not a behaviour change.
    let requirements = if casting_right_half {
        requirements
    } else {
        match casting::spell_mode_selection(state, card_id) {
            Some(ms) => {
                casting::per_mode_target_requirements(&ms, modes_chosen).unwrap_or(requirements)
            }
            None => requirements,
        }
    };
    casting::aura_spell_target_requirements(&eff_chars, requirements)
}

/// CR 602.2b — the target requirements an activated ability announces.
///
/// Reads `calculate_characteristics(state, source)` — **never `card_registry.get()`** —
/// because `ability_index` indexes the *layer-resolved* `activated_abilities` list
/// (`abilities.rs:315-334`); a registry read would bypass Humility/Dress Down removing
/// abilities and Layer-6 `AddActivatedAbility` grants adding them.
///
/// A modal activated ability's per-mode target slice (`abilities.rs:433-458`) is **out of
/// scope here** — this function has no chosen-mode input, so it always returns the
/// printed `ActivatedAbility.targets`. `abilities.rs:433-458` itself hard-rejects
/// combining multiple chosen modes with `ModeSelection.mode_targets`, so the flat list is
/// the correct answer for the single-mode case and an incomplete one (documented, not
/// silently wrong) for the rare multi-mode + `mode_targets` case.
///
/// Missing object or an out-of-range `ability_index` yield `vec![]`.
pub fn ability_target_requirements(
    state: &GameState,
    source: ObjectId,
    ability_index: usize,
) -> Vec<TargetRequirement> {
    let Some(chars) = calculate_characteristics(state, source) else {
        return vec![];
    };
    chars
        .activated_abilities
        .get(ability_index)
        .map(|ab| ab.targets.clone())
        .unwrap_or_default()
}

/// CR 606.3 / CR 601.2c — the target requirements a **loyalty** ability announces.
///
/// # This is deliberately NOT [`ability_target_requirements`], and the difference is the
/// whole reason it exists
///
/// The two functions take the same `(source, ability_index)` argument shape and index
/// **entirely unrelated lists**:
///
/// * [`ability_target_requirements`] indexes `Characteristics::activated_abilities` — the
///   *layer-resolved* activated-ability list, which is what `handle_activate_ability`
///   indexes.
/// * `handle_activate_loyalty_ability` (`rules/engine.rs`, the `loyalty_abilities` binding)
///   indexes `CardDefinition::abilities` **filtered to `AbilityDefinition::LoyaltyAbility`**,
///   read from `state.card_registry()`, and `mtg_simulator::legal_actions` mints the
///   `LegalAction::ActivateLoyaltyAbility { ability_index }` it offers against that same
///   filtered registry list (counting `LoyaltyAbility` entries up to and including the raw
///   index).
///
/// So index 0 means different abilities to the two functions, and on a planeswalker with
/// both activated and loyalty abilities they can name different requirements entirely.
/// `OOS-M11-10(loyalty)`'s own row says "CR 602.2b targets are already reachable through
/// `queries.rs::ability_target_requirements`' sibling path"; that is true of the *machinery*
/// and false of the *index space*, which is why this is a new function rather than a reuse.
///
/// # Why the registry and not `calculate_characteristics`
///
/// Mirroring the handler is the point (the PB-DX20 "one arithmetic, two consumers" shape):
/// the handler reads the registry, so a query that read layer-resolved characteristics could
/// announce a requirement the handler will not validate against, or miss one it will. A
/// planeswalker whose loyalty abilities are altered by a continuous effect would need
/// **both** sides changed together; a divergence here would be silent, and this is the half
/// a client sees.
///
/// Missing object, missing `card_id`, an unregistered card, or an out-of-range
/// `ability_index` all yield `vec![]` — this function never panics and never unwraps.
pub fn loyalty_ability_target_requirements(
    state: &GameState,
    source: ObjectId,
    ability_index: usize,
) -> Vec<TargetRequirement> {
    // NOT `expect_object`: that is the impossible-absence lookup (`state::diagnostics`),
    // which fires a `debug_assert!` and degrades to `None` only in release. Every
    // function in this module promises never to panic, and this one is called with an
    // `ObjectId` a client chose — a CR 400.7-retired id from a stale UI is an ordinary
    // input here, not an engine bug. The first draft used `expect_object` and the
    // batch's own test author caught the contradiction against this doc.
    let Some(obj) = state.objects().get(&source) else {
        return vec![];
    };
    let Some(card_id) = obj.card_id.clone() else {
        return vec![];
    };
    let Some(def) = state.card_registry().get(card_id) else {
        return vec![];
    };
    def.abilities
        .iter()
        .filter_map(|a| match a {
            crate::cards::card_definition::AbilityDefinition::LoyaltyAbility {
                targets, ..
            } => Some(targets),
            _ => None,
        })
        .nth(ability_index)
        .cloned()
        .unwrap_or_default()
}

/// CR 606.4 — the printed loyalty COST of the ability at `ability_index`.
///
/// Added by PB-DX29's `/review` fix cycle (L9): the browser labelled a planeswalker's
/// abilities `"Loyalty ability 0/1/2 of Chandra, Flamecaller"` — three indistinguishable
/// buttons on the very card the batch was dispatched to make usable — and the cost is
/// what a player actually says out loud.
///
/// **It lives here rather than in `tools/play-server` for a reason the batch's own
/// Invariant-7 gate supplied**: formatting the label client-side needed a raw
/// `state.card_registry()` read inside `view.rs`, and
/// `test_ui6_view_rs_reads_game_state_in_exactly_the_three_known_places` immediately went
/// red — correctly, since a new raw `GameState` read in that file is a new
/// hidden-information channel no other Invariant-7 gate can see. Returning the COST and
/// letting the view format it keeps that pin at three and puts the registry read beside
/// the two loyalty queries that already make it.
///
/// Indexing is [`loyalty_ability_target_requirements`]' — see that function's doc for why
/// this is not the activated-ability index space.
pub fn loyalty_ability_cost(
    state: &GameState,
    source: ObjectId,
    ability_index: usize,
) -> Option<crate::cards::card_definition::LoyaltyCost> {
    use crate::cards::card_definition::AbilityDefinition;
    let obj = state.objects().get(&source)?;
    let card_id = obj.card_id.clone()?;
    let def = state.card_registry().get(card_id)?;
    def.abilities
        .iter()
        .filter_map(|a| match a {
            AbilityDefinition::LoyaltyAbility { cost, .. } => Some(cost.clone()),
            _ => None,
        })
        .nth(ability_index)
}

/// CR 606.4 / CR 107.3m — does this loyalty ability's cost carry an `{X}`?
///
/// `LoyaltyCost::MinusX` is the only variant whose paid amount is the activator's choice
/// (`handle_activate_loyalty_ability` reads `x_value.unwrap_or(0)` and both charges that
/// many loyalty counters and stores it as the stack object's `x_value`). Every other
/// variant is a fixed number and a client must not be offered a box for it.
///
/// Indexing and the registry read are [`loyalty_ability_target_requirements`]' — see that
/// function's doc for why this is not the activated-ability index space.
pub fn loyalty_ability_needs_x(state: &GameState, source: ObjectId, ability_index: usize) -> bool {
    use crate::cards::card_definition::{AbilityDefinition, LoyaltyCost};
    // See `loyalty_ability_target_requirements` for why this is not `expect_object`.
    let Some(obj) = state.objects().get(&source) else {
        return false;
    };
    let Some(card_id) = obj.card_id.clone() else {
        return false;
    };
    let Some(def) = state.card_registry().get(card_id) else {
        return false;
    };
    def.abilities
        .iter()
        .filter_map(|a| match a {
            AbilityDefinition::LoyaltyAbility { cost, .. } => Some(cost),
            _ => None,
        })
        .nth(ability_index)
        .is_some_and(|cost| matches!(cost, LoyaltyCost::MinusX))
}

/// Per-slot legal-target candidates, parallel to `requirements`.
///
/// **Advisory only**: each requirement is applied *independently* — inter-target
/// distinctness (`TargetRequirement::TargetPermanentDistinctFrom`, CR 601.2c "another
/// target") is NOT enforced across slots, nor is the collective target-count range.
/// `process_command` remains authoritative and a target from this list can still be
/// rejected at cast time (e.g. by `enforce_inter_target_distinctness`, `casting.rs:6192`).
///
/// Candidates are delegated to `casting::validate_targets_inner` one at a time — this
/// function never re-derives hexproof/shroud/protection/type-restriction legality itself,
/// exactly the delegation the M11-local session plan §1 fact 4 requires.
///
/// Candidate objects are drawn from `ZoneId::Battlefield`, `ZoneId::Stack`, and
/// `ZoneId::Graveyard(_)` — the exact union of zones any arm of
/// `casting::validate_object_satisfies_requirement` (`casting.rs:6253`+) can accept.
/// `TargetCreature`/`TargetPermanent`/`TargetArtifact`/etc. require `Battlefield`;
/// `TargetSpell`/`TargetSpellWithFilter`/`TargetSpellOrAbilityWithSingleTarget`/
/// `TargetSpellWithSingleTarget` require `Stack`; `TargetCardInYourGraveyard`/
/// `TargetCardInGraveyard` require a `Graveyard(_)`. No arm accepts any other zone, so
/// this is a narrowing that only skips candidates the validator would reject anyway — kept
/// so a full multiplayer game's hand/library/exile contents are not characteristic-
/// resolved on every UI query.
///
/// Candidate players are every player in `state.turn.turn_order` still in the game (the
/// same liveness test `trigger_target_candidates` uses, `abilities.rs:7525-7530`).
///
/// Iteration order is deterministic: players in seat order, then objects in ascending
/// `ObjectId` order (`state.objects()` is an `imbl::OrdMap`) — players pushed first,
/// matching `trigger_target_candidates`'s ordering (`abilities.rs:7496`).
///
/// Hidden information (Architecture Invariant 7): the three enumerated zones are the
/// public ones, and the return value is bare `ObjectId`/`PlayerId` with no
/// characteristics attached, so this cannot leak a hand or a library order.
///
/// Cost: one `validate_targets_inner` per (requirement × candidate), and each object
/// candidate costs a `calculate_characteristics`. On a full four-player board (~100
/// objects across the three zones) a two-slot spell is ~200 layer computations. Fine
/// per user action; **measure before Session 5 wires this to an endpoint a browser
/// polls**, and cache per `(state, source)` there rather than making this lazier.
pub fn legal_targets_per_slot(
    state: &GameState,
    caster: PlayerId,
    source: ObjectId,
    requirements: &[TargetRequirement],
) -> Vec<Vec<Target>> {
    let source_chars = calculate_characteristics(state, source);

    let mut candidates: Vec<Target> = Vec::new();
    for &p in state.turn.turn_order.iter() {
        if let Some(pl) = state.expect_player(p) {
            if !pl.has_lost && !pl.has_conceded {
                candidates.push(Target::Player(p));
            }
        }
    }
    for (id, obj) in state.objects().iter() {
        if matches!(
            obj.zone,
            ZoneId::Battlefield | ZoneId::Stack | ZoneId::Graveyard(_)
        ) {
            candidates.push(Target::Object(*id));
        }
    }

    // PB-DX52 (`OOS-DX25b-1`) -- the ABILITY half of "target spell or ability".
    //
    // An activated or triggered ability's stack entry is never in `state.objects` (it
    // owns no card), so the walk above structurally cannot reach it and Bolt Bend's
    // printed "or ability" half was dead. `Target::StackObject` names the entry by its
    // own `StackObject::id`.
    //
    // **Only entries that own NO card in `ZoneId::Stack` are offered, and the reason is
    // de-duplication, not legality.** A spell's entry is already offered above, by the
    // card id `casting.rs::handle_cast_spell` moved into `ZoneId::Stack` (CR 601.2a) --
    // that is the canonical announcement, the one every existing probe and golden script
    // uses. Offering the same spell a second time under its entry id would put two
    // candidates on the wire for one game object and make the browser's target picker
    // show a duplicate row. `validate_stack_object_satisfies_requirement` still ACCEPTS
    // an entry id for `TargetSpell`/`TargetSpellWithSingleTarget`, so nothing legal is
    // refused -- only the OFFER is de-duplicated.
    //
    // Consequence, stated rather than left to be discovered: a COPY of a spell is still
    // not offered, because `copy.rs` clones the original's `kind` wholesale and
    // `card_in_stack_zone` therefore returns `Some` for it. `OOS-DX25b-2` stays open by
    // this rule rather than by accident. A copy of an ABILITY *is* offered -- its cloned
    // kind owns no card either -- which is CR 707.10b-correct.
    //
    // Order: after the objects, ascending by stack position (bottom of the stack first),
    // which is `state.stack_objects`' own `imbl::Vector` order and therefore
    // deterministic. `retarget::retarget_candidates` appends the identical set in the
    // identical order; the R6 gate there asserts the two universes are equal by
    // execution.
    for so in state.stack_objects.iter() {
        if crate::state::stack_registry::card_in_stack_zone(&so.kind).is_none() {
            candidates.push(Target::StackObject(so.id));
        }
    }

    requirements
        .iter()
        .map(|req| {
            candidates
                .iter()
                .filter(|cand| {
                    casting::validate_targets_inner(
                        state,
                        std::slice::from_ref(cand),
                        std::slice::from_ref(req),
                        caster,
                        source_chars.as_ref(),
                        Some(source),
                    )
                    .is_ok()
                })
                .cloned()
                .collect()
        })
        .collect()
}

/// (min, max) target count for a requirement list (CR 601.2c).
pub fn target_count_range(requirements: &[TargetRequirement]) -> (usize, usize) {
    casting::target_count_range(requirements)
}

/// CR 508.1h — the unflattened attack-tax total a candidate `attackers` declaration
/// would owe, in the canonical pip order `Command::DeclareAttackers::hybrid_choices`/
/// `phyrexian_life_payments` index against.
///
/// The turn-face-up cost is knowable from the card def alone; the attack-tax cost is
/// **not** knowable outside the engine, because it is a function of the declared
/// attacker set, the live restriction list and the source permanents' zones —
/// `LegalAction::DeclareAttackers { eligible, targets }` carries no attacker set at
/// all (the set is chosen later, client-side). Without this query, every client
/// would have to re-implement the CR 508.1h accumulation itself, which is precisely
/// the drift class OOS-RS-2 was.
///
/// **Anti-drift guarantee**: this delegates to `combat::accumulate_attack_tax_total`,
/// the exact same function `handle_declare_attackers`'s own validation calls — see
/// its doc for the full canonical-order contract (copy-major: defenders ascending by
/// `PlayerId`, then one complete copy of the defender's per-creature cost per
/// attacking creature, then restrictions in `state.restrictions` order within a
/// copy). Two independent copies of that order is how OOS-RS2-1/OOS-DP4-1 happened
/// in the first place; this function and the validation block share one.
///
/// Returns `None` when the total is `ManaCost::default()` (no tax applies to this
/// declaration) — never `Some(ManaCost::default())`. `player` is accepted for API
/// symmetry with the command this advises and so a future caller-side validation (a
/// declared attacker not controlled by `player`, a self-attack) has a place to live
/// without a signature change; the accumulation itself does not need it today, since
/// `attackers` already encodes which creatures and targets are being asked about —
/// `handle_declare_attackers` re-validates attacker ownership independently before
/// this figure would ever be charged (Architecture Invariant 3: this is advisory
/// only, never authoritative).
///
/// **`None` does NOT always mean "this declaration is free" — read this before
/// treating an absent total as "nothing to pay" (PB-DX6 fix-cycle Finding 7,
/// OOS-DX6-1).** `combat::accumulate_attack_tax_total` excludes any restriction whose
/// `cost_per_creature` carries `x_count > 0` from the returned total entirely (X has
/// no announcement channel on `Command::DeclareAttackers`, CR 107.3/601.2b), and for a
/// defender whose ONLY restriction is such an X tax this function returns `None`
/// exactly as it would for a genuinely untaxed defender. `handle_declare_attackers`
/// will nonetheless hard-reject any declaration engaging that defender. So `None`
/// means one of two different things a caller cannot distinguish from this return
/// value alone: "no restriction applies here" (the declaration is legal and free), or
/// "an X-only restriction applies here" (the declaration will be rejected regardless
/// of what is paid). A mixed total (a payable restriction plus a separate X
/// restriction on the same defender) is worse: `Some` is returned, but it silently
/// omits the X restriction's own contribution, so even a `Some` total is not
/// necessarily the FULL reason a declaration might be rejected. Do not build a payment
/// plan from `None` and assume it will be accepted — see `params.rs`'s call site for
/// the current SR-38 residue this causes. Widening the signature to express "X applies
/// and is unrepresentable" is OOS-DX6-1's job, not this batch's.
pub fn attack_tax_total(
    state: &GameState,
    player: PlayerId,
    attackers: &[(ObjectId, AttackTarget)],
) -> Option<ManaCost> {
    let _ = player;
    let total = combat::accumulate_attack_tax_total(state, attackers);
    if total == ManaCost::default() {
        None
    } else {
        Some(total)
    }
}

/// CR 702.52a/b: every card in `player`'s graveyard that could replace a draw
/// right now, as `(card, N)`, sorted by `ObjectId` for determinism.
///
/// (Placement: appended at the END of this module, not inserted above
/// `attack_tax_total`. A doc comment attaches to the item immediately
/// following it, so inserting a function between an existing doc block and
/// its function silently reassigns that doc to the newcomer and leaves the
/// original undocumented. That is exactly what happened on this function's
/// first draft — `attack_tax_total` lost its whole doc, including the
/// load-bearing PB-DX6 "`None` does NOT always mean free" warning — and it is
/// the same trap `replacement::perform_remaining_draws`' own placement note
/// records from PB-DX2's fix cycle. Second occurrence; append here.)
///
/// **Advisory, re-validated at answer time** (this module's header): the
/// caller still submits a `Command::ChooseDredge` through the normal path,
/// and `handle_choose_dredge`'s own `Some(id)` validation
/// (`rules::replacement`) re-checks both conjuncts against LIVE state at the
/// moment of the answer, not against whatever this function returned when
/// the offer was computed — a card can leave the graveyard, or the library
/// can shrink below `n`, between the offer and the answer.
///
/// **One derivation, two consumers (PB-DX23 §3 Q2, the PB-DX20 shape):**
/// `rules::replacement::check_would_draw_replacement` calls this SAME
/// function for the `DredgeChoiceRequired` offer's own `options` list rather
/// than keeping its own copy of the CR 702.52a/b scan — re-deriving dredge
/// eligibility inside `crates/simulator` (the other consumer, via this
/// query) would be the `OOS-RS-2` drift class this module's whole point is
/// to avoid. A differential probe between the two consumers proves they
/// AGREE, not that either is CR-correct — see `dredge_options`'s own direct
/// CR 702.52a/b test pair for the correctness half (PB-DX20's durable
/// lesson).
// PB-DX23 review, finding E3: this reads `obj.characteristics.keywords`
// RAW, not layer-resolved (`calculate_characteristics`). That matches
// `handle_choose_dredge`'s own answer-time validator, which reads the same
// raw field — so there is no offer-vs-engine divergence today — but it means
// an effect that altered a graveyard card's keywords (e.g. "cards in
// graveyards lose all abilities") would be invisible to BOTH sides at once
// rather than caught by either. Closing this means changing this function
// and `handle_choose_dredge` together, not just here (the PB-DX19 durable
// lesson: a differential probe between two raw readers proves agreement, not
// correctness).
pub fn dredge_options(state: &GameState, player: PlayerId) -> Vec<(ObjectId, u32)> {
    let graveyard_zone = ZoneId::Graveyard(player);
    let library_zone = ZoneId::Library(player);
    // SR-14: the library zone is built before turn 1 and never removed (ground truth 2).
    let library_count = state
        .expect_zone(&library_zone)
        .map(|z| z.len())
        .unwrap_or(0);
    let mut options: Vec<(ObjectId, u32)> = state
        .objects()
        .values()
        .filter(|obj| obj.zone == graveyard_zone)
        .filter_map(|obj| {
            obj.characteristics.keywords.iter().find_map(|kw| {
                if let KeywordAbility::Dredge(n) = kw {
                    if (*n as usize) <= library_count {
                        Some((obj.id, *n))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
        .collect();
    // Sort for determinism (by ObjectId).
    options.sort_by_key(|(id, _)| *id);
    options
}

/// CR 702.140a (PB-DX50, `OOS-DX25-1`) — the battlefield permanents `caster` may
/// legally name as the mutate host when casting `card` with the mutate alternative
/// cost, in ascending `ObjectId` order.
///
/// **This exists because the offer layer had a FOURTH copy of the predicate, and it was
/// the wrong one.** `StubProvider` (`crates/simulator/src/legal_actions.rs`) built its
/// mutate offers from a hand-rolled `owner == player && card_types.contains(Creature)
/// && !subtypes.contains("Human")` filter that read `o.characteristics` **RAW** rather
/// than layer-resolved, so it was blind to the layer system entirely: a creature
/// animated by a continuous effect was invisible to it, and a creature turned into a
/// Human by a type-changing effect was still offered. With PB-DX50 tightening the cast
/// path with the full CR 115 target-legality machinery (hexproof CR 702.11b, shroud
/// CR 702.18a, protection CR 702.16b), leaving that copy in place would have shipped an
/// SR-38 defect — a clean offer followed by a guaranteed refusal, which is the exact
/// shape PB-DX29 gated Fuse to avoid, PB-DX44 re-created while fixing it, and PB-DX45
/// shipped and had to fix. This batch would have been the fourth.
///
/// Delegates the whole decision to `legal_targets_per_slot` over
/// `casting::mutate_target_requirement()` — the SAME requirement `handle_cast_spell`
/// appends and the SAME `validate_targets_inner` it validates with — so no CR 115 logic
/// is duplicated outside the engine.
///
/// `card` is the mutate card itself (in hand), passed as the `source` so that
/// protection-from-source qualities (CR 702.16b) are evaluated against the spell that
/// would be cast, exactly as at cast time.
///
/// Advisory only (module doc): a value returned here can still be rejected by
/// `handle_cast_spell`, which re-validates independently.
pub fn legal_mutate_hosts(state: &GameState, caster: PlayerId, card: ObjectId) -> Vec<ObjectId> {
    let per_slot = legal_targets_per_slot(
        state,
        caster,
        card,
        std::slice::from_ref(&casting::mutate_target_requirement()),
    );
    per_slot
        .into_iter()
        .next()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|t| match t {
            Target::Object(id) => Some(id),
            Target::Player(_) => None,
            // PB-DX52: CR 702.140a's mutate host is a non-Human creature on the
            // battlefield; a stack entry can never satisfy `mutate_target_requirement`,
            // so this arm is unreachable by construction rather than by convention.
            Target::StackObject(_) => None,
        })
        .collect()
}

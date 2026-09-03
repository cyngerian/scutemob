//! CR 714 — the Saga query.
//!
//! Every CR 714 decision the engine makes ("is this a Saga", "what is its final chapter
//! number", "which chapter abilities does it still have") used to be taken independently at
//! five sites, each by reading the **printed** card definition
//! (`def.effective_abilities(obj.is_transformed)`) and none by consulting the layer axis.
//! A permanent whose abilities are blanked therefore kept accruing lore counters
//! (CR 714.3b), kept firing chapter triggers (CR 714.2b) and was sacrificed anyway
//! (CR 714.4) — it behaved exactly as if nothing had happened to it.
//!
//! This module answers all of those questions once, from one place, so the five sites
//! cannot disagree. It is a **read-only query**: it takes `&GameState` and returns a plain
//! struct. Nothing here is a `Command`, an `Effect` payload or a `GameEvent` field, which is
//! why it moves neither wire fingerprint.
//!
//! ## Why this is not lowered into `Characteristics`
//!
//! The obvious alternative is to lower `AbilityDefinition::SagaChapter` into
//! `Characteristics` and let the layer walk blank it like any other ability. That would move
//! **both** `PROTOCOL_SCHEMA_FINGERPRINT` and `HASH_SCHEMA_FINGERPRINT` (`Characteristics` is
//! a PROTOCOL closure root and a hashed type) for a question five call sites ask. The
//! continuous-effect scan in [`crate::rules::layers::abilities_are_blanked`] answers the same
//! question with no type change at all, and is what this module delegates to.
//!
//! ## What is deliberately NOT a consumer
//!
//! CR 113.7a — *an ability on the stack exists independently of its source*. `resolution.rs`
//! resolves a chapter ability that has **already triggered**, and blanking the Saga after the
//! trigger went on the stack neither counters nor changes it. Those two sites keep reading
//! the printed def and say so in source; do not "finish the job" by wiring them here.

use crate::cards::card_definition::AbilityDefinition;
use crate::state::game_object::ObjectId;
use crate::state::GameState;

/// The CR 714 view of one object, taken after the layer axis has been consulted.
///
/// Built by [`saga_view`]. Cheap enough to call per-object inside an SBA sweep: it is one
/// registry lookup plus one pass over the active continuous effects.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SagaView {
    /// `(ability_index, chapter)` for each chapter ability the permanent **retains** after
    /// the layer axis is consulted.
    ///
    /// Indices are into `effective_abilities(obj.is_transformed)` — the same index space
    /// every CardDef ability-index consumer resolves against (CR 712.8d/e, PB-RS4). They are
    /// the *enumeration* indices of the full ability list, not positions within a filtered
    /// chapter-only list, because that is the namespace a `PendingTrigger.ability_index`
    /// lives in.
    ///
    /// Empty when the permanent's abilities are blanked (CR 613.1f / CR 305.7 / CR 708.2a),
    /// and empty when it was never a Saga.
    pub chapters: Vec<(usize, u32)>,
    /// CR 714.3a Saga-ness — "as a Saga enters the battlefield, its controller puts a lore
    /// counter on it". This is **not** the same question as [`SagaView::has_chapters`], and
    /// the difference is load-bearing at exactly one site; see [`saga_view`].
    ///
    /// False for a face-down permanent (CR 708.2a: no text, **no subtypes** — so it is not a
    /// Saga at all).
    pub is_saga_permanent: bool,
}

impl SagaView {
    /// CR 714.2d: *"A Saga's final chapter number is the greatest value among chapter
    /// abilities it has. If a Saga somehow has no chapter abilities, its final chapter
    /// number is 0."*
    ///
    /// Returns `None` when there are no retained chapter abilities. `None` is deliberately
    /// distinct from `Some(0)`: CR 714.4 applies to *"a Saga permanent with one or more
    /// chapter abilities"*, so a permanent with none is outside that rule entirely rather
    /// than being one whose threshold is trivially met.
    pub fn final_chapter(&self) -> Option<u32> {
        self.chapters.iter().map(|(_, ch)| *ch).max()
    }

    /// CR 714.3b / CR 714.4's *"with one or more chapter abilities"* clause.
    pub fn has_chapters(&self) -> bool {
        !self.chapters.is_empty()
    }

    /// Is `i` — an `ability_index` in the `effective_abilities` namespace — one of the
    /// chapter abilities this permanent still has? Used by CR 714.4's "chapter ability has
    /// triggered but not yet left the stack" guard.
    pub fn is_chapter_index(&self, i: usize) -> bool {
        self.chapters.iter().any(|(idx, _)| *idx == i)
    }
}

/// Build the CR 714 view of `id`.
///
/// Derivation, in order:
///
/// - object absent, no `card_id`, or the id is unregistered → the empty view.
/// - `printed` = the `AbilityDefinition::SagaChapter` entries of
///   `def.effective_abilities(obj.is_transformed)`, each carrying its **enumeration** index
///   (CR 712.8d/e — only the face that is actually showing can make this permanent a Saga).
/// - `is_saga_permanent` = there are printed chapters **and** the permanent is not face-down.
/// - `chapters` = empty if [`crate::rules::layers::abilities_are_blanked`], else `printed`.
///
/// Both blanking channels answer this one query, and they cannot disagree: face-down implies
/// `abilities_are_blanked`, so a face-down permanent is neither a Saga nor a holder of
/// chapters.
///
/// ## The two questions are not the same question
///
/// `is_saga_permanent` and `has_chapters()` come apart under a Layer-6 `RemoveAllAbilities`,
/// and CR 714 asks for different ones at different sites:
///
/// - **CR 714.3a** — *"As a Saga **without the read ahead ability** enters the battlefield,
///   its controller puts a lore counter on it."* There is **no** "with one or more chapter
///   abilities" clause here. A blanked permanent keeps its subtypes (CR 613.1f removes
///   abilities, not types), so it **is** still a Saga and **does** get the counter.
/// - **CR 714.3b** and **CR 714.4** both say *"with one or more chapter abilities"*
///   explicitly, so both read `has_chapters()`.
/// - **CR 714.2b** needs the ability to exist at the moment counters are put on, so the
///   chapter list is what decides which triggers fire — none, while blanked.
///
/// # Stated residual (seeded, deliberately not fixed here)
///
/// `is_saga_permanent` uses *printed chapter abilities* as the Saga-ness proxy — the same
/// proxy all five sites already used — rather than the layer-resolved `SubType("Saga")`. A
/// type-setting effect that strips the Saga subtype **without** blanking abilities
/// (`imprisoned_in_the_moon`'s `SetTypeLine`) would leave the proxy saying "Saga" where
/// CR 205.3h says otherwise. Population is 0 at the only site that reads it (CR 714.3a runs
/// as a permanent *enters*, and no corpus effect attaches an Aura at that instant). Widening
/// the query into `calculate_characteristics` is **not** the fix — that is the lowering the
/// design mandate rejects, and it would reopen `OOS-SIM2-6`'s recursion from an SBA sweep.
pub fn saga_view(state: &GameState, id: ObjectId) -> SagaView {
    // CR 400.7: a caller's id may name a permanent that already left. That is a fizzle, and
    // a departed object has no abilities to trigger.
    let Some(obj) = state.fizzle_object(id) else {
        return SagaView::default();
    };
    let Some(cid) = obj.card_id.as_ref() else {
        return SagaView::default();
    };
    let Some(def) = state.card_registry.get(cid.clone()) else {
        return SagaView::default();
    };
    // CR 712.8d/e (PB-RS4): the index space is the currently-visible face's *effective*
    // ability list, and the index carried out of here is the enumeration index into it.
    let printed: Vec<(usize, u32)> = def
        .effective_abilities(obj.is_transformed)
        .iter()
        .enumerate()
        .filter_map(|(i, a)| match a {
            AbilityDefinition::SagaChapter { chapter, .. } => Some((i, *chapter)),
            _ => None,
        })
        .collect();
    // Short-circuit BEFORE the continuous-effect scan. This is not an optimisation with a
    // behavioural caveat, it is an identity: `chapters` is either `printed` or empty, and
    // `is_saga_permanent` requires `!printed.is_empty()`, so a def with no printed chapters
    // yields the default view down both arms whatever the layer axis says. Retained chapters
    // are a SUBSET of printed ones — the query can only ever remove.
    //
    // It matters because `check_saga_sbas` asks this question about **every phased-in
    // battlefield permanent on every SBA check**, and `abilities_are_blanked` clones the
    // object's `Characteristics` and walks every active continuous effect. Without this line
    // the CR 714 query would put that cost on every creature, land and artifact in the game
    // to answer a question only Sagas can answer yes to.
    if printed.is_empty() {
        return SagaView::default();
    }
    // CR 708.2a: a face-down permanent has no subtypes, so it is not a Saga. Same spelling
    // as `layers::abilities_are_blanked`'s channel-1 conjunct, deliberately.
    let face_down = obj.status.face_down && obj.face_down_as.is_some();
    let is_saga_permanent = !printed.is_empty() && !face_down;
    let chapters = if crate::rules::layers::abilities_are_blanked(state, id) {
        Vec::new()
    } else {
        printed
    };
    SagaView {
        chapters,
        is_saga_permanent,
    }
}

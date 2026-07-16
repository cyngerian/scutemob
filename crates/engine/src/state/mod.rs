//! Game state model: zones, objects, players, and the core GameState struct.
//!
//! All state uses `im` persistent data structures for structural sharing,
//! enabling cheap snapshots and deterministic replay.
pub mod ability_definition_registry;
pub mod builder;
pub mod diagnostics;
pub mod error;
pub mod hash;
pub mod keyword_registry;
/// Escape hatches for tests — see the module docs. Not compiled in production builds.
#[cfg(any(test, feature = "test-util"))]
pub mod test_util;
pub mod turn;

// The pure-data half of the state model lives in `mtg-card-types`, below this
// crate, so that card definitions can be built without a running game. These
// module re-exports keep every `crate::state::game_object::…` path in the
// engine resolving exactly as it did when the files were here. Anything that
// reads or mutates a live `GameState` stays in this crate.
pub use mtg_card_types::state::{
    combat, continuous_effect, dungeon, game_object, player, replacement_effect, stack, stubs,
    targeting, types, zone,
};

// Re-export primary types for convenient access via `use mtg_engine::state::*`
use crate::cards::CardRegistry;
use crate::rules::events::GameEvent;
pub use builder::{
    register_commander_zone_replacements, GameStateBuilder, ObjectSpec, PlayerBuilder,
};
pub use error::GameStateError;
use imbl::{OrdMap, Vector};
pub use mtg_card_types::state::dungeon::get_dungeon;
pub use mtg_card_types::state::{
    AbilityInstance, ActivatedAbility, ActivationCost, ActiveRestriction, AdditionalCost,
    AdditionalLandPlaySource, AffinityTarget, AltCostKind, AttackTarget, BlockingExceptionFilter,
    CardId, CardType, ChampionFilter, Characteristics, Color, CombatState, ContinuousEffect,
    CounterType, CumulativeUpkeepCost, DamageTargetFilter, DayNight, DeathTriggerFilter,
    DelayedTrigger, Designations, DungeonDef, DungeonId, DungeonState, ETBSuppressFilter,
    ETBSuppressor, ETBTriggerFilter, EffectDuration, EffectFilter, EffectId, EffectLayer,
    EnchantControllerConstraint, EnchantFilter, EnchantTarget, FaceDownKind, FlashGrant,
    FlashGrantFilter, GameObject, GameRestriction, HybridMana, HybridManaPayment, InterveningIf,
    KeywordAbility, LandwalkType, LayerModification, ManaAbility, ManaColor, ManaCost, ManaPool,
    MergedComponent, ObjectFilter, ObjectId, ObjectStatus, PendingTrigger, PendingZoneChange,
    PhyrexianMana, PlayFromGraveyardPermission, PlayFromTopFilter, PlayFromTopPermission,
    PlayerFilter, PlayerId, PlayerState, ProtectionQuality, ReplacementEffect, ReplacementId,
    ReplacementModification, ReplacementTrigger, RoomDef, RoomIndex, SacrificeFilter, SpellTarget,
    StackObject, StackObjectKind, SubType, SuperType, Target, TriggerData, TriggerDoubler,
    TriggerDoublerFilter, TriggerEvent, TriggeredAbilityDef, TurnFaceUpMethod, UpkeepCostKind,
    Zone, ZoneId, ZoneType,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
pub use turn::{Phase, Step, TurnState};
/// The complete state of an MTG game at a point in time.
///
/// Uses `im` persistent data structures for O(1) cloning via structural sharing.
/// All state transitions produce new `GameState` values; old values are retained
/// for undo, replay, and "what if" analysis.
///
/// See architecture doc Section 2.1 for the full field listing and rationale.
///
/// # Sealed (SR-3)
///
/// Every field is `pub(crate)`. From outside the engine crate a `GameState` is a
/// read-only view: one accessor per field, and no way to write. The only
/// sanctioned mutation path is submitting a
/// [`Command`](crate::rules::commands::Command) to `process_command`. That is
/// what makes architecture invariant #3 — *"there is no way to change game state
/// except through the Command enum"* — enforced by the compiler instead of by
/// review. Everything downstream (networking, replay, rewind, deterministic
/// tests) depends on the command/event log being the complete story of a game;
/// a stray `state.objects.insert(..)` would silently corrupt that history.
///
/// Reading is unrestricted:
///
/// ```
/// fn count_objects(state: &mtg_engine::GameState) -> usize {
///     state.objects().len()
/// }
/// ```
///
/// Writing does not compile outside the engine — this doctest is a regression
/// guard on the seal, and fails the build if a field is ever made `pub` again:
///
/// ```compile_fail
/// fn tamper(state: &mut mtg_engine::GameState) {
///     // error[E0616]: field `objects` of struct `GameState` is private
///     state.objects.clear();
/// }
/// ```
///
/// The `test-util` escape hatches ([`crate::state::test_util`], plus the
/// `*_mut()` accessors) are a separate matter, and this doctest cannot guard
/// them: doctests are compiled under the test profile, where cargo's feature
/// unification has already switched `test-util` on. `cargo build --workspace`
/// is what proves no production consumer depends on a hatch, because it does not
/// build dev-dependencies. It runs in CI for exactly that reason.
///
/// ## What the seal does not cover
///
/// Three ways to obtain an arbitrary `GameState` remain open by design. All
/// three yield an *owned* state rather than a handle for mutating a live one,
/// so none of them can corrupt a game already in progress:
///
/// - [`GameStateBuilder`] — the documented public constructor.
/// - `#[derive(Deserialize)]` — required by replay and networking, which load
///   states from disk and from the wire. A caller can deserialize any state it
///   can describe; that is inherent to accepting serialized state at all.
/// - [`crate::testing::replay_harness::build_initial_state`] — builds a state
///   from a game script. It is `pub` and ships in production builds.
///
/// The invariant this seal protects is narrower than "no one can name a bad
/// state": it is that a `GameState` handed to a consumer cannot be edited behind
/// the command/event log's back, so the log remains the complete history of a
/// game and rewind/replay stay sound.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    /// Current turn/phase/step/priority state.
    pub(crate) turn: TurnState,
    /// All players indexed by PlayerId.
    pub(crate) players: OrdMap<PlayerId, PlayerState>,
    /// All zones indexed by ZoneId.
    pub(crate) zones: OrdMap<ZoneId, Zone>,
    /// All game objects indexed by ObjectId.
    pub(crate) objects: OrdMap<ObjectId, GameObject>,
    /// Active continuous effects (CR 611). Applied via the layer system.
    pub(crate) continuous_effects: Vector<ContinuousEffect>,
    /// Delayed triggers waiting for conditions (CR 603.7).
    pub(crate) delayed_triggers: Vector<DelayedTrigger>,
    /// Active replacement effects (CR 614).
    pub(crate) replacement_effects: Vector<ReplacementEffect>,
    /// Monotonic counter for generating ReplacementIds.
    pub(crate) next_replacement_id: u64,
    /// Zone changes waiting for player choice among replacement effects (CR 616.1).
    /// SBA loop skips objects with pending entries; resolved by `OrderReplacements`.
    pub(crate) pending_zone_changes: Vector<PendingZoneChange>,
    /// Commanders awaiting the owner's zone-return choice (CR 903.9a).
    ///
    /// Each entry is `(owner, object_id)`. The SBA skips commanders already in
    /// this list so the choice event is not re-emitted every SBA pass.
    /// Cleared when the owner sends `ReturnCommanderToCommandZone` or
    /// `LeaveCommanderInZone`.
    pub(crate) pending_commander_zone_choices: Vector<(PlayerId, ObjectId)>,
    /// Prevention shield counters: remaining capacity for `PreventDamage(n)` effects (CR 615.7).
    /// Keyed by ReplacementId. When a counter reaches 0 the corresponding ReplacementEffect
    /// is removed from `replacement_effects`. `PreventAllDamage` effects need no counter.
    pub(crate) prevention_counters: imbl::OrdMap<ReplacementId, u32>,
    /// Triggered abilities waiting to be put on the stack.
    pub(crate) pending_triggers: Vector<PendingTrigger>,
    /// Active trigger-doubling effects (Panharmonicon-style, CR 603.2d).
    ///
    /// When a trigger that matches any doubler's filter is about to be queued,
    /// it is queued `additional_triggers` additional times. Entries are added
    /// when a permanent with a trigger-doubling ability enters the battlefield
    /// and removed when that permanent leaves.
    pub(crate) trigger_doublers: Vector<TriggerDoubler>,
    /// Active ETB trigger suppression effects (Torpor Orb-style, CR 614.16a).
    ///
    /// When a creature matching any suppressor's filter would have its ETB triggered
    /// ability queued, the trigger is skipped entirely (it never fires). Entries are
    /// added when a permanent with `AbilityDefinition::SuppressCreatureETBTriggers`
    /// enters the battlefield and cleaned up when that permanent leaves.
    #[serde(default)]
    pub(crate) etb_suppressors: Vector<ETBSuppressor>,
    /// Active game restrictions (stax effects, CR 604).
    ///
    /// Static abilities that prevent players from taking certain actions (casting spells,
    /// attacking, activating abilities). Checked at action-legality time in casting.rs
    /// and combat.rs. Registered when a permanent with `AbilityDefinition::StaticRestriction`
    /// enters the battlefield; cleaned up when that permanent leaves.
    #[serde(default)]
    pub(crate) restrictions: Vector<ActiveRestriction>,
    /// Active flash grants (cast-as-though-flash permissions, CR 601.3b).
    ///
    /// Allows players to cast certain spells at instant speed. Checked at casting-validation
    /// time in casting.rs. Registered by Effect::GrantFlash or AbilityDefinition::StaticFlashGrant.
    /// Cleaned up by duration expiry or source leaving battlefield.
    #[serde(default)]
    pub(crate) flash_grants: Vector<FlashGrant>,
    /// Active play-from-top-of-library permissions (CR 601.3, CR 305.1).
    ///
    /// Allows players to play lands and/or cast spells from the top of their library.
    /// Registered by AbilityDefinition::StaticPlayFromTop when the source permanent enters.
    /// Cleaned up when the source permanent leaves the battlefield (in reset_turn_state).
    #[serde(default)]
    pub(crate) play_from_top_permissions: Vector<crate::state::stubs::PlayFromTopPermission>,
    /// Active play-from-graveyard permissions (CR 601.3, CR 305.1).
    ///
    /// Allows players to play lands and/or cast spells from their graveyard.
    /// Registered by AbilityDefinition::StaticPlayFromGraveyard when the source permanent
    /// enters the battlefield, or by CreateEmblem with play_from_graveyard filter.
    /// Cleaned up when the source permanent leaves the battlefield. Emblem-sourced
    /// permissions are permanent (emblems never leave the command zone).
    #[serde(default)]
    pub(crate) play_from_graveyard_permissions:
        Vector<crate::state::stubs::PlayFromGraveyardPermission>,
    /// Stack objects (spells and abilities on the stack).
    pub(crate) stack_objects: Vector<StackObject>,
    /// CR 113.7a / 608.2h / 702.80c / 702.90e: last-known-information snapshots of
    /// damage sources that left the battlefield, keyed by the object's (now-retired)
    /// `ObjectId`.
    ///
    /// The stored `GameObject`'s `characteristics` field holds the object's
    /// **layer-resolved** characteristics as they last existed on the battlefield
    /// (not the usual base characteristics), captured via `calculate_characteristics`
    /// immediately before the object was removed. This lets a damage source that has
    /// changed zones (CR 400.7) still apply wither / infect / deathtouch / lifelink
    /// when its damage ability finally resolves, because "the wither/infect rules
    /// function no matter what zone an object … deals damage from" (CR 702.80c /
    /// 702.90e) and the effect must use the source's last known information
    /// (CR 608.2h / 113.7a) once it is no longer in its expected zone.
    ///
    /// Populated in `move_object_to_zone` / `move_object_to_bottom_of_zone`; cleared
    /// when the stack empties (`handle_all_passed`). `ObjectId`s are monotonic
    /// (`next_object_id`), so a stale entry can never be matched by a different object.
    ///
    /// SR-24: only a departing permanent whose layer-resolved characteristics carry one
    /// of the four consumed keywords (wither / infect / deathtouch / lifelink) is stored
    /// — the sole readers (`effects::damage_source_characteristics` /
    /// `damage_source_controller`) test for exactly those, so a keyword-less permanent's
    /// snapshot could never be observed. Board wipes therefore no longer clone every
    /// vanilla creature into this map. See `capture_lki_snapshot`.
    #[serde(default)]
    pub(crate) lki_objects: OrdMap<ObjectId, GameObject>,
    /// Current combat state, if in a combat phase.
    pub(crate) combat: Option<CombatState>,
    /// Monotonic counter for generating ObjectIds and timestamps.
    pub(crate) timestamp_counter: u64,
    /// Tracks state hash occurrences for mandatory infinite loop detection (CR 104.4b).
    ///
    /// Maps a truncated game-state hash (u64) to the number of times that hash has
    /// been seen during the current mandatory-action sequence (SBA + trigger cycles).
    /// Reset whenever a player makes a meaningful game choice.
    ///
    /// Excluded from `public_state_hash` — this is metadata, not game state.
    /// See `rules/loop_detection.rs` for the detection algorithm.
    pub(crate) loop_detection_hashes: imbl::OrdMap<u64, u32>,
    /// Append-only event log for triggers that look back at history.
    pub(crate) history: Vector<GameEvent>,
    /// CR 702.69a: Global count of permanents put into a graveyard from the battlefield
    /// this turn, across all players and including tokens.
    ///
    /// Used by Gravestorm to determine how many copies to create. Incremented in
    /// `move_object_to_zone` / `move_object_to_bottom_of_zone` when `from ==
    /// ZoneId::Battlefield` and `to == ZoneId::Graveyard(_)`. Reset by
    /// `reset_turn_state` at the start of each turn.
    ///
    /// Tokens count: they briefly exist in the graveyard (CR 704.5d) before ceasing to
    /// exist as an SBA. The increment happens before any subsequent SBA check.
    #[serde(default)]
    pub(crate) permanents_put_into_graveyard_this_turn: u32,
    /// CR 702.30a: Pending echo payment choices.
    ///
    /// When a KeywordTrigger (Echo) resolves, the controller must choose to pay or sacrifice.
    /// The game pauses until a `Command::PayEcho` is received for each entry.
    /// Each entry is `(player, permanent_id, echo_cost)`.
    ///
    /// Only one echo payment can be pending at a time (triggers resolve one at a time
    /// from the stack), but using `Vector` is consistent with other pending-choice patterns.
    #[serde(default)]
    pub(crate) pending_echo_payments: imbl::Vector<(PlayerId, ObjectId, ManaCost)>,
    /// CR 702.24a: Pending cumulative upkeep payment choices.
    ///
    /// When a KeywordTrigger (CumulativeUpkeep) resolves (after adding the age counter), the
    /// controller must choose to pay or sacrifice. The game pauses until a
    /// `Command::PayCumulativeUpkeep` is received for each entry.
    /// Each entry is `(player, permanent_id, per_counter_cost)`.
    #[serde(default)]
    pub(crate) pending_cumulative_upkeep_payments: imbl::Vector<(
        PlayerId,
        ObjectId,
        crate::state::types::CumulativeUpkeepCost,
    )>,
    /// CR 702.59a: Pending recover payment choices.
    ///
    /// When a RecoverTrigger resolves, the controller must choose to pay the
    /// recover cost or exile the card. The game pauses until a
    /// `Command::PayRecover` is received for each entry.
    /// Each entry is `(player, recover_card_id, recover_cost)`.
    #[serde(default)]
    pub(crate) pending_recover_payments: imbl::Vector<(PlayerId, ObjectId, ManaCost)>,
    /// CR 702.57b: Cards that have activated their forecast ability this turn.
    ///
    /// Keyed by CardId (not ObjectId) since the card stays in hand and retains
    /// its identity. Reset at the start of each turn in `reset_turn_state`.
    /// Each forecast ability can be activated at most once per turn (CR 702.57b).
    #[serde(default)]
    pub(crate) forecast_used_this_turn: imbl::OrdSet<crate::state::player::CardId>,
    /// CR 730.1: Current day/night designation of the game.
    ///
    /// `None` = neither day nor night (game start, default).
    /// `Some(Day)` = it is currently day.
    /// `Some(Night)` = it is currently night.
    ///
    /// Once set, never returns to None (CR 730.1: "the game will have exactly one
    /// of those designations from that point forward").
    ///
    /// Checked and potentially changed in the untap step (CR 730.2).
    /// Also set immediately when a Daybound or Nightbound permanent enters the
    /// battlefield while neither day nor night (CR 702.145d/g).
    #[serde(default)]
    pub(crate) day_night: Option<DayNight>,
    /// CR 730.2: The number of spells cast by the previous turn's active player.
    ///
    /// Captured at the end of each turn (in `reset_turn_state`) from the active
    /// player's `spells_cast_this_turn`. Used at the next turn's untap step to
    /// determine if day/night should change:
    /// - Day → Night if previous player cast 0 spells (CR 730.2a)
    /// - Night → Day if previous player cast 2+ spells (CR 730.2b)
    #[serde(default)]
    pub(crate) previous_turn_spells_cast: u32,
    /// CR 309.4: Per-player dungeon tracking.
    ///
    /// Maps `PlayerId` → `DungeonState` for each player currently exploring a dungeon.
    /// An entry exists while the player has a dungeon in their command zone (CR 309.3).
    /// The entry is removed when the dungeon is completed and removed from the game (CR 309.7).
    ///
    /// Empty at game start — no player has a dungeon in the command zone.
    #[serde(default)]
    pub(crate) dungeon_state: OrdMap<PlayerId, dungeon::DungeonState>,
    /// CR 725.1: Which player currently has the initiative.
    ///
    /// `None` = no player has the initiative (game start, or initiative was never taken).
    /// `Some(player_id)` = that player has the initiative.
    ///
    /// Only one player can have the initiative at a time (CR 725.3). Taking the
    /// initiative also causes the taker to venture into The Undercity (CR 725.2).
    #[serde(default)]
    pub(crate) has_initiative: Option<PlayerId>,
    /// CR 724.1: The monarch is a designation a player can have.
    ///
    /// `None` = no player is the monarch (game start, or monarch left the game
    /// and no replacement could be found — CR 724.4).
    /// `Some(player_id)` = that player is the monarch.
    ///
    /// Only one player can be the monarch at a time (CR 724.3).
    /// Inherent triggers (CR 724.2): EOT draw + combat damage steals.
    #[serde(default)]
    pub(crate) monarch: Option<PlayerId>,
    /// CR 305.2: Static "additional land play" sources from permanents on the battlefield.
    ///
    /// Each entry records a permanent that grants its controller one or more extra land
    /// plays per turn. Applied in `reset_turn_state` to increment `land_plays_remaining`.
    /// Cleaned up when the source permanent leaves the battlefield.
    #[serde(default)]
    pub(crate) additional_land_play_sources:
        imbl::Vector<crate::state::stubs::AdditionalLandPlaySource>,
    /// CR 615.1: When true, all combat damage is prevented for the rest of the turn.
    ///
    /// Set by Effect::PreventAllCombatDamage. Reset in `reset_turn_state` at turn start.
    #[serde(default)]
    pub(crate) prevent_all_combat_damage: bool,
    /// CR 615: Objects whose combat damage OUTPUT is prevented this turn.
    ///
    /// Set by Effect::PreventCombatDamageFromOrTo with `prevent_from: true`.
    /// Reset in `reset_turn_state` at turn start.
    #[serde(default)]
    pub(crate) combat_damage_prevented_from: imbl::OrdSet<ObjectId>,
    /// CR 615: Objects whose combat damage INPUT is prevented this turn.
    ///
    /// Set by Effect::PreventCombatDamageFromOrTo with `prevent_to: true`.
    /// Reset in `reset_turn_state` at turn start.
    #[serde(default)]
    pub(crate) combat_damage_prevented_to: imbl::OrdSet<ObjectId>,
    /// Card definitions registry: maps CardId → CardDefinition.
    ///
    /// Static data, never changes during a game. Held as `Arc` so state clones
    /// share the registry without copying it. Excluded from state hashing and
    /// serialization (reconstructed from the card database on load).
    #[serde(skip)]
    pub(crate) card_registry: Arc<CardRegistry>,
}
/// Read-only accessors for every [`GameState`] field.
///
/// The fields themselves are `pub(crate)` (SR-3). Outside the engine crate,
/// `GameState` is an immutable view: the only sanctioned way to change it is to
/// submit a [`Command`](crate::rules::commands::Command) through
/// `process_command`, which is what makes architecture invariant #3 ("all player
/// actions are Commands") a machine guarantee rather than a convention.
///
/// Mutable access exists only behind the `test-util` feature — see the
/// [`escape hatch`](GameState#escape-hatches) block below.
impl GameState {
    /// Read-only access to the `turn` field.
    pub fn turn(&self) -> &TurnState {
        &self.turn
    }

    /// Read-only access to the `players` field.
    pub fn players(&self) -> &OrdMap<PlayerId, PlayerState> {
        &self.players
    }

    /// Read-only access to the `zones` field.
    pub fn zones(&self) -> &OrdMap<ZoneId, Zone> {
        &self.zones
    }

    /// Read-only access to the `objects` field.
    pub fn objects(&self) -> &OrdMap<ObjectId, GameObject> {
        &self.objects
    }

    /// Read-only access to the `continuous_effects` field.
    pub fn continuous_effects(&self) -> &Vector<ContinuousEffect> {
        &self.continuous_effects
    }

    /// Read-only access to the `delayed_triggers` field.
    pub fn delayed_triggers(&self) -> &Vector<DelayedTrigger> {
        &self.delayed_triggers
    }

    /// Read-only access to the `replacement_effects` field.
    pub fn replacement_effects(&self) -> &Vector<ReplacementEffect> {
        &self.replacement_effects
    }

    /// Read-only access to the `pending_zone_changes` field.
    pub fn pending_zone_changes(&self) -> &Vector<PendingZoneChange> {
        &self.pending_zone_changes
    }

    /// Read-only access to the `pending_commander_zone_choices` field.
    pub fn pending_commander_zone_choices(&self) -> &Vector<(PlayerId, ObjectId)> {
        &self.pending_commander_zone_choices
    }

    /// Read-only access to the `prevention_counters` field.
    pub fn prevention_counters(&self) -> &imbl::OrdMap<ReplacementId, u32> {
        &self.prevention_counters
    }

    /// Read-only access to the `pending_triggers` field.
    pub fn pending_triggers(&self) -> &Vector<PendingTrigger> {
        &self.pending_triggers
    }

    /// Read-only access to the `trigger_doublers` field.
    pub fn trigger_doublers(&self) -> &Vector<TriggerDoubler> {
        &self.trigger_doublers
    }

    /// Read-only access to the `etb_suppressors` field.
    pub fn etb_suppressors(&self) -> &Vector<ETBSuppressor> {
        &self.etb_suppressors
    }

    /// Read-only access to the `restrictions` field.
    pub fn restrictions(&self) -> &Vector<ActiveRestriction> {
        &self.restrictions
    }

    /// Read-only access to the `flash_grants` field.
    pub fn flash_grants(&self) -> &Vector<FlashGrant> {
        &self.flash_grants
    }

    /// Read-only access to the `play_from_top_permissions` field.
    pub fn play_from_top_permissions(&self) -> &Vector<PlayFromTopPermission> {
        &self.play_from_top_permissions
    }

    /// Read-only access to the `play_from_graveyard_permissions` field.
    pub fn play_from_graveyard_permissions(&self) -> &Vector<PlayFromGraveyardPermission> {
        &self.play_from_graveyard_permissions
    }

    /// Read-only access to the `stack_objects` field.
    pub fn stack_objects(&self) -> &Vector<StackObject> {
        &self.stack_objects
    }

    /// Read-only access to the `combat` field.
    pub fn combat(&self) -> &Option<CombatState> {
        &self.combat
    }

    /// Read-only access to the whole `lki_objects` last-known-information store.
    ///
    /// SR-3 one-public-read-accessor convention: `lki_objects` is `pub(crate)`, and
    /// [`lki_object_snapshot`](GameState::lki_object_snapshot) is a `pub(crate)` keyed
    /// getter, so nothing outside the engine could observe the LKI store at all — the
    /// replay viewer, in particular, cannot render a state's captured snapshots. This
    /// exposes the map by shared reference (read-only; the only mutation paths remain
    /// `capture_lki_snapshot` and `maybe_clear_lki_objects`). See the `lki_objects`
    /// field docs for what a snapshot means and when it is captured/cleared.
    pub fn lki_objects(&self) -> &OrdMap<ObjectId, GameObject> {
        &self.lki_objects
    }

    /// CR 113.7a / 608.2h: last-known-information snapshot of an object that has left
    /// the battlefield, keyed by its retired `ObjectId`. Returns `None` if no snapshot
    /// was captured for `id` (the object never left the battlefield with the stack
    /// non-empty, or the snapshot was already cleared). The returned object's
    /// `characteristics` are layer-resolved as they last existed on the battlefield.
    /// See the `lki_objects` field docs.
    pub(crate) fn lki_object_snapshot(&self, id: ObjectId) -> Option<&GameObject> {
        self.lki_objects.get(&id)
    }

    /// SR-13: drop last-known-information snapshots once nothing can reference them —
    /// the stack AND the pending-trigger queue are both empty, so no on-stack ability and
    /// no about-to-be-flushed trigger can still need a departed source's LKI. Safe to
    /// call at any priority or turn boundary; a no-op unless both queues are drained. See
    /// the `lki_objects` field docs.
    ///
    /// Known limitation: a *delayed* triggered ability whose source died in an earlier
    /// priority window and which deals damage across the gap (e.g. "at the beginning of
    /// the next end step, ~ deals damage") loses its snapshot when the stack empties in
    /// between, and would fall back to normal damage. This is not a regression (no LKI
    /// existed before SR-13) and is vanishingly rare; the common "damage ability on the
    /// stack / pending when the source dies" case is fully covered.
    pub(crate) fn maybe_clear_lki_objects(&mut self) {
        if !self.lki_objects.is_empty()
            && self.stack_objects.is_empty()
            && self.pending_triggers.is_empty()
        {
            self.lki_objects = OrdMap::new();
        }
    }

    /// Read-only access to the `loop_detection_hashes` field.
    pub fn loop_detection_hashes(&self) -> &imbl::OrdMap<u64, u32> {
        &self.loop_detection_hashes
    }

    /// Read-only access to the `history` field.
    pub fn history(&self) -> &Vector<GameEvent> {
        &self.history
    }

    /// Read-only access to the `pending_echo_payments` field.
    pub fn pending_echo_payments(&self) -> &imbl::Vector<(PlayerId, ObjectId, ManaCost)> {
        &self.pending_echo_payments
    }

    /// Read-only access to the `pending_cumulative_upkeep_payments` field.
    pub fn pending_cumulative_upkeep_payments(
        &self,
    ) -> &imbl::Vector<(PlayerId, ObjectId, CumulativeUpkeepCost)> {
        &self.pending_cumulative_upkeep_payments
    }

    /// Read-only access to the `pending_recover_payments` field.
    pub fn pending_recover_payments(&self) -> &imbl::Vector<(PlayerId, ObjectId, ManaCost)> {
        &self.pending_recover_payments
    }

    /// Read-only access to the `forecast_used_this_turn` field.
    pub fn forecast_used_this_turn(&self) -> &imbl::OrdSet<CardId> {
        &self.forecast_used_this_turn
    }

    /// Read-only access to the `dungeon_state` field.
    pub fn dungeon_state(&self) -> &OrdMap<PlayerId, DungeonState> {
        &self.dungeon_state
    }

    /// Read-only access to the `additional_land_play_sources` field.
    pub fn additional_land_play_sources(&self) -> &imbl::Vector<AdditionalLandPlaySource> {
        &self.additional_land_play_sources
    }

    /// Read-only access to the `combat_damage_prevented_from` field.
    pub fn combat_damage_prevented_from(&self) -> &imbl::OrdSet<ObjectId> {
        &self.combat_damage_prevented_from
    }

    /// Read-only access to the `combat_damage_prevented_to` field.
    pub fn combat_damage_prevented_to(&self) -> &imbl::OrdSet<ObjectId> {
        &self.combat_damage_prevented_to
    }

    /// Read-only access to the `card_registry` field.
    pub fn card_registry(&self) -> &Arc<CardRegistry> {
        &self.card_registry
    }

    /// Read-only access to the `next_replacement_id` field.
    ///
    /// Named `_counter` to avoid colliding with the `next_replacement_id()`
    /// generator method, which allocates a fresh id.
    pub fn next_replacement_id_counter(&self) -> u64 {
        self.next_replacement_id
    }

    /// Read-only access to the `timestamp_counter` field.
    pub fn timestamp_counter(&self) -> u64 {
        self.timestamp_counter
    }

    /// Read-only access to the `permanents_put_into_graveyard_this_turn` field.
    pub fn permanents_put_into_graveyard_this_turn(&self) -> u32 {
        self.permanents_put_into_graveyard_this_turn
    }

    /// Read-only access to the `previous_turn_spells_cast` field.
    pub fn previous_turn_spells_cast(&self) -> u32 {
        self.previous_turn_spells_cast
    }

    /// Read-only access to the `day_night` field.
    pub fn day_night(&self) -> Option<DayNight> {
        self.day_night
    }

    /// Read-only access to the `has_initiative` field.
    pub fn has_initiative(&self) -> Option<PlayerId> {
        self.has_initiative
    }

    /// Read-only access to the `monarch` field.
    pub fn monarch(&self) -> Option<PlayerId> {
        self.monarch
    }

    /// Read-only access to the `prevent_all_combat_damage` field.
    pub fn prevent_all_combat_damage(&self) -> bool {
        self.prevent_all_combat_damage
    }
}

/// # Escape hatches
///
/// Mutable access to [`GameState`] internals, compiled **only** under
/// `cfg(test)` or the `test-util` cargo feature.
///
/// ## Why this exists
///
/// Tests and benchmarks must be able to construct arbitrary mid-game positions
/// (a creature with damage marked, a specific combat state, a stack mid-resolution)
/// that no legal sequence of `Command`s can reach cheaply. That is a legitimate
/// need, but it must not be reachable from shipping code, or architecture
/// invariant #3 degrades back into a convention.
///
/// ## The guarantee
///
/// `test-util` is off in any production build. The engine's own integration
/// tests and benches turn it on via a self dev-dependency in `Cargo.toml`:
///
/// ```toml
/// [dev-dependencies]
/// mtg-engine = { path = ".", features = ["test-util"] }
/// ```
///
/// `cargo build --workspace` is the gate that proves the seal: it does not build
/// dev-dependencies, so `test-util` is off and none of these methods exist. If a
/// production consumer (tui, replay-viewer, simulator, network) ever reaches for
/// one, that build fails.
///
/// **Caveat:** under `--all-targets` (i.e. `cargo test --all`, `cargo clippy
/// --all-targets`) cargo unifies features across the workspace, so `test-util`
/// *is* enabled for every crate in that profile. Those commands therefore cannot
/// detect a production consumer using an escape hatch — only
/// `cargo build --workspace` can. Keep it in the gate list.
///
/// ## Preferred alternatives
///
/// Reach for [`GameStateBuilder`] first: it is the documented, always-public
/// constructor for setting up a game position. These hatches are for the cases
/// it cannot express.
#[cfg(any(test, feature = "test-util"))]
impl GameState {
    /// Escape hatch: mutable access to `turn`. See [module docs](GameState#escape-hatches).
    pub fn turn_mut(&mut self) -> &mut TurnState {
        &mut self.turn
    }

    /// Escape hatch: mutable access to `players`. See [module docs](GameState#escape-hatches).
    pub fn players_mut(&mut self) -> &mut OrdMap<PlayerId, PlayerState> {
        &mut self.players
    }

    /// Escape hatch: mutable access to `zones`. See [module docs](GameState#escape-hatches).
    pub fn zones_mut(&mut self) -> &mut OrdMap<ZoneId, Zone> {
        &mut self.zones
    }

    /// Escape hatch: mutable access to `objects`. See [module docs](GameState#escape-hatches).
    pub fn objects_mut(&mut self) -> &mut OrdMap<ObjectId, GameObject> {
        &mut self.objects
    }

    /// Escape hatch: mutable access to `continuous_effects`. See [module docs](GameState#escape-hatches).
    pub fn continuous_effects_mut(&mut self) -> &mut Vector<ContinuousEffect> {
        &mut self.continuous_effects
    }

    /// Escape hatch: mutable access to `delayed_triggers`. See [module docs](GameState#escape-hatches).
    pub fn delayed_triggers_mut(&mut self) -> &mut Vector<DelayedTrigger> {
        &mut self.delayed_triggers
    }

    /// Escape hatch: mutable access to `replacement_effects`. See [module docs](GameState#escape-hatches).
    pub fn replacement_effects_mut(&mut self) -> &mut Vector<ReplacementEffect> {
        &mut self.replacement_effects
    }

    /// Escape hatch: mutable access to `pending_zone_changes`. See [module docs](GameState#escape-hatches).
    pub fn pending_zone_changes_mut(&mut self) -> &mut Vector<PendingZoneChange> {
        &mut self.pending_zone_changes
    }

    /// Escape hatch: mutable access to `pending_commander_zone_choices`. See [module docs](GameState#escape-hatches).
    pub fn pending_commander_zone_choices_mut(&mut self) -> &mut Vector<(PlayerId, ObjectId)> {
        &mut self.pending_commander_zone_choices
    }

    /// Escape hatch: mutable access to `prevention_counters`. See [module docs](GameState#escape-hatches).
    pub fn prevention_counters_mut(&mut self) -> &mut imbl::OrdMap<ReplacementId, u32> {
        &mut self.prevention_counters
    }

    /// Escape hatch: mutable access to `pending_triggers`. See [module docs](GameState#escape-hatches).
    pub fn pending_triggers_mut(&mut self) -> &mut Vector<PendingTrigger> {
        &mut self.pending_triggers
    }

    /// Escape hatch: mutable access to `trigger_doublers`. See [module docs](GameState#escape-hatches).
    pub fn trigger_doublers_mut(&mut self) -> &mut Vector<TriggerDoubler> {
        &mut self.trigger_doublers
    }

    /// Escape hatch: mutable access to `etb_suppressors`. See [module docs](GameState#escape-hatches).
    pub fn etb_suppressors_mut(&mut self) -> &mut Vector<ETBSuppressor> {
        &mut self.etb_suppressors
    }

    /// Escape hatch: mutable access to `restrictions`. See [module docs](GameState#escape-hatches).
    pub fn restrictions_mut(&mut self) -> &mut Vector<ActiveRestriction> {
        &mut self.restrictions
    }

    /// Escape hatch: mutable access to `flash_grants`. See [module docs](GameState#escape-hatches).
    pub fn flash_grants_mut(&mut self) -> &mut Vector<FlashGrant> {
        &mut self.flash_grants
    }

    /// Escape hatch: mutable access to `play_from_top_permissions`. See [module docs](GameState#escape-hatches).
    pub fn play_from_top_permissions_mut(&mut self) -> &mut Vector<PlayFromTopPermission> {
        &mut self.play_from_top_permissions
    }

    /// Escape hatch: mutable access to `play_from_graveyard_permissions`. See [module docs](GameState#escape-hatches).
    pub fn play_from_graveyard_permissions_mut(
        &mut self,
    ) -> &mut Vector<PlayFromGraveyardPermission> {
        &mut self.play_from_graveyard_permissions
    }

    /// Escape hatch: mutable access to `stack_objects`. See [module docs](GameState#escape-hatches).
    pub fn stack_objects_mut(&mut self) -> &mut Vector<StackObject> {
        &mut self.stack_objects
    }

    /// Escape hatch: mutable access to `combat`. See [module docs](GameState#escape-hatches).
    pub fn combat_mut(&mut self) -> &mut Option<CombatState> {
        &mut self.combat
    }

    /// Escape hatch: mutable access to `loop_detection_hashes`. See [module docs](GameState#escape-hatches).
    pub fn loop_detection_hashes_mut(&mut self) -> &mut imbl::OrdMap<u64, u32> {
        &mut self.loop_detection_hashes
    }

    /// Escape hatch: mutable access to `history`. See [module docs](GameState#escape-hatches).
    pub fn history_mut(&mut self) -> &mut Vector<GameEvent> {
        &mut self.history
    }

    /// Escape hatch: mutable access to `pending_echo_payments`. See [module docs](GameState#escape-hatches).
    pub fn pending_echo_payments_mut(
        &mut self,
    ) -> &mut imbl::Vector<(PlayerId, ObjectId, ManaCost)> {
        &mut self.pending_echo_payments
    }

    /// Escape hatch: mutable access to `pending_cumulative_upkeep_payments`. See [module docs](GameState#escape-hatches).
    pub fn pending_cumulative_upkeep_payments_mut(
        &mut self,
    ) -> &mut imbl::Vector<(PlayerId, ObjectId, CumulativeUpkeepCost)> {
        &mut self.pending_cumulative_upkeep_payments
    }

    /// Escape hatch: mutable access to `pending_recover_payments`. See [module docs](GameState#escape-hatches).
    pub fn pending_recover_payments_mut(
        &mut self,
    ) -> &mut imbl::Vector<(PlayerId, ObjectId, ManaCost)> {
        &mut self.pending_recover_payments
    }

    /// Escape hatch: mutable access to `forecast_used_this_turn`. See [module docs](GameState#escape-hatches).
    pub fn forecast_used_this_turn_mut(&mut self) -> &mut imbl::OrdSet<CardId> {
        &mut self.forecast_used_this_turn
    }

    /// Escape hatch: mutable access to `dungeon_state`. See [module docs](GameState#escape-hatches).
    pub fn dungeon_state_mut(&mut self) -> &mut OrdMap<PlayerId, DungeonState> {
        &mut self.dungeon_state
    }

    /// Escape hatch: mutable access to `additional_land_play_sources`. See [module docs](GameState#escape-hatches).
    pub fn additional_land_play_sources_mut(
        &mut self,
    ) -> &mut imbl::Vector<AdditionalLandPlaySource> {
        &mut self.additional_land_play_sources
    }

    /// Escape hatch: mutable access to `combat_damage_prevented_from`. See [module docs](GameState#escape-hatches).
    pub fn combat_damage_prevented_from_mut(&mut self) -> &mut imbl::OrdSet<ObjectId> {
        &mut self.combat_damage_prevented_from
    }

    /// Escape hatch: mutable access to `combat_damage_prevented_to`. See [module docs](GameState#escape-hatches).
    pub fn combat_damage_prevented_to_mut(&mut self) -> &mut imbl::OrdSet<ObjectId> {
        &mut self.combat_damage_prevented_to
    }

    /// Escape hatch: mutable access to `card_registry`. See [module docs](GameState#escape-hatches).
    pub fn card_registry_mut(&mut self) -> &mut Arc<CardRegistry> {
        &mut self.card_registry
    }

    /// Escape hatch: mutable access to `next_replacement_id`.
    ///
    /// Note the asymmetry with the reader `next_replacement_id_counter()`: only the
    /// reader needs the `_counter` suffix, to avoid colliding with the
    /// `next_replacement_id()` id-generator method.
    /// See [module docs](GameState#escape-hatches).
    pub fn next_replacement_id_mut(&mut self) -> &mut u64 {
        &mut self.next_replacement_id
    }

    /// Escape hatch: mutable access to `timestamp_counter`. See [module docs](GameState#escape-hatches).
    pub fn timestamp_counter_mut(&mut self) -> &mut u64 {
        &mut self.timestamp_counter
    }

    /// Escape hatch: mutable access to `permanents_put_into_graveyard_this_turn`. See [module docs](GameState#escape-hatches).
    pub fn permanents_put_into_graveyard_this_turn_mut(&mut self) -> &mut u32 {
        &mut self.permanents_put_into_graveyard_this_turn
    }

    /// Escape hatch: mutable access to `previous_turn_spells_cast`. See [module docs](GameState#escape-hatches).
    pub fn previous_turn_spells_cast_mut(&mut self) -> &mut u32 {
        &mut self.previous_turn_spells_cast
    }

    /// Escape hatch: mutable access to `day_night`. See [module docs](GameState#escape-hatches).
    pub fn day_night_mut(&mut self) -> &mut Option<DayNight> {
        &mut self.day_night
    }

    /// Escape hatch: mutable access to `has_initiative`. See [module docs](GameState#escape-hatches).
    pub fn has_initiative_mut(&mut self) -> &mut Option<PlayerId> {
        &mut self.has_initiative
    }

    /// Escape hatch: mutable access to `monarch`. See [module docs](GameState#escape-hatches).
    pub fn monarch_mut(&mut self) -> &mut Option<PlayerId> {
        &mut self.monarch
    }

    /// Escape hatch: mutable access to `prevent_all_combat_damage`. See [module docs](GameState#escape-hatches).
    pub fn prevent_all_combat_damage_mut(&mut self) -> &mut bool {
        &mut self.prevent_all_combat_damage
    }
}

impl GameState {
    /// Generates the next unique ObjectId, incrementing the timestamp counter.
    pub(crate) fn next_object_id(&mut self) -> ObjectId {
        self.timestamp_counter += 1;
        ObjectId(self.timestamp_counter)
    }
    /// Returns the current timestamp value (for continuous effect ordering).
    pub fn current_timestamp(&self) -> u64 {
        self.timestamp_counter
    }
    /// Generates the next unique ReplacementId, incrementing the counter.
    pub(crate) fn next_replacement_id(&mut self) -> ReplacementId {
        let id = ReplacementId(self.next_replacement_id);
        self.next_replacement_id += 1;
        id
    }
    /// Look up a player by ID.
    pub fn player(&self, id: PlayerId) -> Result<&PlayerState, GameStateError> {
        self.players
            .get(&id)
            .ok_or(GameStateError::PlayerNotFound(id))
    }
    /// Look up a mutable player by ID.
    pub(crate) fn player_mut(&mut self, id: PlayerId) -> Result<&mut PlayerState, GameStateError> {
        self.players
            .get_mut(&id)
            .ok_or(GameStateError::PlayerNotFound(id))
    }
    /// Look up a game object by ID.
    pub fn object(&self, id: ObjectId) -> Result<&GameObject, GameStateError> {
        self.objects
            .get(&id)
            .ok_or(GameStateError::ObjectNotFound(id))
    }
    /// Look up a mutable game object by ID.
    pub(crate) fn object_mut(&mut self, id: ObjectId) -> Result<&mut GameObject, GameStateError> {
        self.objects
            .get_mut(&id)
            .ok_or(GameStateError::ObjectNotFound(id))
    }
    /// Look up a zone by ID.
    pub fn zone(&self, id: &ZoneId) -> Result<&Zone, GameStateError> {
        self.zones.get(id).ok_or(GameStateError::ZoneNotFound(*id))
    }
    /// Add a new game object to a zone, assigning it a fresh ObjectId and timestamp.
    /// Returns the assigned ObjectId.
    pub(crate) fn add_object(
        &mut self,
        mut object: GameObject,
        zone_id: ZoneId,
    ) -> Result<ObjectId, GameStateError> {
        let id = self.next_object_id();
        object.id = id;
        object.zone = zone_id;
        object.timestamp = self.timestamp_counter;
        // For tokens (is_token=true) entering the battlefield, set entered_turn.
        // Tokens always enter this turn — unlike permanents placed via builder.rs
        // (which use None to represent "pre-existing before current turn").
        if zone_id == ZoneId::Battlefield && object.is_token {
            object.entered_turn = Some(self.turn.turn_number);
            // PB-AC6 / CR 111.10: mark that this player created a token this turn.
            // Single chokepoint -- every GameEvent::TokenCreated emission site funnels
            // a token GameObject through add_object before emitting.
            if let Some(ps) = self.players.get_mut(&object.controller) {
                ps.created_token_this_turn = true;
            }
        }
        // Add to zone — MR-M1-01/MR-M1-04: single access, no redundant guard.
        let zone = self
            .zones
            .get_mut(&zone_id)
            .ok_or(GameStateError::ZoneNotFound(zone_id))?;
        zone.insert(id);
        // Add to objects map
        self.objects.insert(id, object);
        Ok(id)
    }
    /// SR-13: capture a last-known-information snapshot of an object about to leave a
    /// zone, so a damage ability still on the stack can read its source's wither /
    /// infect / deathtouch / lifelink after the source is gone.
    ///
    /// CR 702.80c / 702.90e: wither and infect "function no matter what zone an object …
    /// deals damage from." CR 608.2h / 113.7a: an effect that needs information about
    /// its source (here, the source's keywords) uses the source's last known
    /// information once it is no longer in its expected zone.
    ///
    /// Only battlefield-leaves are captured. The snapshot's `characteristics` are
    /// **layer-resolved** (via `calculate_characteristics` while the object is still
    /// present), so keywords granted by continuous effects (e.g. Tainted Strike's
    /// infect, Basilisk Collar's deathtouch/lifelink) are preserved, matching the live
    /// read path. `next_object_id` is monotonic, so the retired id can never collide
    /// with a future object; the map is cleared once nothing can reference it — the
    /// stack and pending-trigger queue are both empty (`maybe_clear_lki_objects`, called
    /// from `rules::engine::handle_all_passed` and `reset_turn_state`).
    ///
    /// SR-24: within battlefield-leaves, the snapshot is *stored* only when the
    /// layer-resolved characteristics carry a keyword the store's readers actually
    /// consult (wither / infect / deathtouch / lifelink) — see the inline note. Two
    /// properties make that safe:
    ///
    /// - It is **not** gated on a non-empty stack. A damage ability can be a *pending*
    ///   trigger (queued, not yet on the stack), or not yet queued at all, when its
    ///   source dies to the same event — a creature dealt lethal combat damage whose
    ///   "deals damage" trigger fires from that damage, or a "when this dies, it deals
    ///   damage" trigger created only after the death move. At the SBA that kills it the
    ///   stack (and possibly the pending queue) is empty. Keying on the departing
    ///   object's own keywords, not on queue state, covers every such case.
    /// - A permanent carrying none of the four keywords is invisible to both readers, so
    ///   skipping its snapshot changes no outcome — only the map's transient contents.
    ///
    /// Must be called BEFORE the object is removed from `self.objects` (it reads the
    /// object's live layer-resolved characteristics), and — SR-23 — AFTER the move's
    /// error checks (source zone exists and contains the object), so a move that errors
    /// out never leaves a ghost snapshot for an object that is still on the battlefield.
    fn capture_lki_snapshot(&mut self, object_id: ObjectId, from: ZoneId, old_object: &GameObject) {
        if from != ZoneId::Battlefield {
            return;
        }
        if let Some(chars) = crate::rules::layers::calculate_characteristics(self, object_id) {
            // SR-24: the `lki_objects` store is read by exactly two consumers
            // (`damage_source_characteristics` / `damage_source_controller` in
            // `effects/mod.rs`), and each reads it *only* to test the departed source for
            // one of four damage keywords — Wither (CR 702.80c), Infect (CR 702.90e),
            // Deathtouch (CR 702.2), Lifelink (CR 702.15b) — via CR 608.2h / 113.7a. A
            // permanent whose layer-resolved characteristics carry none of them can never
            // change an outcome through this store, so cloning it in is pure waste that the
            // next `maybe_clear_lki_objects` discards. On a board wipe that is *every*
            // vanilla creature, token and land. Gate the clone+insert on keyword presence.
            //
            // The gate keys on the *layer-resolved* `chars`, not `old_object`'s base
            // keywords, because a relevant keyword can be granted by a continuous effect
            // (Tainted Strike's infect, Basilisk Collar's deathtouch) — invisible before
            // layers — so `calculate_characteristics` is still required. It is also
            // timing-independent: it does not consult the stack or pending-trigger queues,
            // so a source that dies with both empty and whose "when this dies, deal damage"
            // trigger is queued only afterward still gets its snapshot iff it has the
            // keyword. This changes the *contents* of `lki_objects` for keyword-less
            // departures but touches no `HashInto` impl and no serde shape, so neither
            // SR-17 fingerprint moves and `HASH_SCHEMA_VERSION` does not bump (measured
            // numbers + compatibility reasoning: `docs/sr-24-lki-capture-cost.md`).
            //
            // COUPLING: this set must equal the keywords the two readers in
            // `effects/mod.rs` consult on a snapshot — `damage_source_characteristics`
            // (wither/infect/deathtouch) and `damage_source_controller` (lifelink).
            // Removing one here reddens `tests/primitives/sr13_lki_damage_source.rs`
            // (that source's effect stops applying from a dead source). Adding a *new*
            // reader that consults a fifth snapshot keyword is NOT machine-caught: add
            // it here and add a matching sr13 case, or the gate will silently drop those
            // snapshots.
            const LKI_RELEVANT_KEYWORDS: [KeywordAbility; 4] = [
                KeywordAbility::Wither,
                KeywordAbility::Infect,
                KeywordAbility::Deathtouch,
                KeywordAbility::Lifelink,
            ];
            if !LKI_RELEVANT_KEYWORDS
                .iter()
                .any(|kw| chars.keywords.contains(kw))
            {
                return;
            }
            let mut snapshot = old_object.clone();
            snapshot.characteristics = chars;
            self.lki_objects.insert(object_id, snapshot);
        }
    }
    /// Move a game object from its current zone to a new zone.
    ///
    /// Implements CR 400.7: "An object that moves from one zone to another becomes
    /// a new object with no memory of, or relation to, its previous existence."
    ///
    /// The old ObjectId is retired. A new ObjectId is assigned. Status, counters,
    /// attachments, and controller are reset. Returns the new ObjectId and a
    /// snapshot of the old object (for trigger processing).
    pub(crate) fn move_object_to_zone(
        &mut self,
        object_id: ObjectId,
        to: ZoneId,
    ) -> Result<(ObjectId, GameObject), GameStateError> {
        // MR-M0-13: Validate destination zone exists BEFORE any mutation.
        // Previously the source was removed before checking the destination,
        // leaving the state corrupt if the destination zone was missing.
        if !self.zones.contains_key(&to) {
            return Err(GameStateError::ZoneNotFound(to));
        }
        // Look up the current object
        let old_object = self
            .objects
            .get(&object_id)
            .ok_or(GameStateError::ObjectNotFound(object_id))?
            .clone();
        let from = old_object.zone;
        // SR-23: run *every* error check before capturing LKI, so an errored move
        // (missing/mismatched source zone) leaves no ghost snapshot for an object that
        // is still live. Validate the source zone exists and contains the object first,
        // via a shared borrow, then capture, then remove. The object is still present
        // in `self.objects` and still in its battlefield zone at capture time — exactly
        // as before — so `capture_lki_snapshot` sees identical layer-resolved
        // characteristics and the success-path state hash is unchanged.
        let from_zone = self
            .zones
            .get(&from)
            .ok_or(GameStateError::ZoneNotFound(from))?;
        if !from_zone.contains(&object_id) {
            return Err(GameStateError::ObjectNotInZone(object_id, from));
        }
        // SR-13: snapshot last-known information before the object is removed, so a
        // damage ability still on the stack can read its source's keywords once the
        // source is gone (CR 113.7a / 608.2h / 702.80c / 702.90e).
        self.capture_lki_snapshot(object_id, from, &old_object);
        // Remove from old zone. Membership was just verified above and nothing has
        // mutated the zone since, so this cannot fail.
        let removed = self
            .zones
            .get_mut(&from)
            .expect("source zone existence checked above")
            .remove(&object_id);
        debug_assert!(removed, "membership checked above, cannot fail");
        // Remove old object from objects map
        self.objects.remove(&object_id);
        // Create new object with fresh ID (CR 400.7)
        let new_id = self.next_object_id();
        let mut new_object = GameObject {
            triggered_abilities_fired_this_turn: imbl::OrdSet::new(),
            id: new_id,
            card_id: old_object.card_id.clone(),
            characteristics: old_object.characteristics.clone(),
            controller: old_object.owner, // Controller resets to owner
            owner: old_object.owner,
            zone: to,
            status: ObjectStatus::default(),
            counters: OrdMap::new(),
            attachments: Vector::new(),
            attached_to: None,
            damage_marked: 0,
            deathtouch_damage: false,
            is_token: old_object.is_token,
            is_emblem: old_object.is_emblem,
            timestamp: self.timestamp_counter,
            // CR 302.6: a permanent entering the battlefield has summoning sickness
            // until the beginning of its controller's next untap step.
            has_summoning_sickness: to == ZoneId::Battlefield,
            // Track which turn this permanent entered the battlefield.
            // Used by Neriv's "entered this turn" filter. None for non-battlefield zones
            // (field is cleared on zone change per CR 400.7).
            entered_turn: if to == ZoneId::Battlefield {
                Some(self.turn.turn_number)
            } else {
                None
            },
            // CR 400.7: goad state is not preserved across zone changes.
            goaded_by: imbl::Vector::new(),
            // CR 400.7: kicked status is not preserved across zone changes
            // (a permanent re-entering is not kicked).
            kicker_times_paid: 0,
            // CR 400.7: alt-cost status (evoke/escape/dash) is not preserved across zone changes.
            cast_alt_cost: None,
            foretold_turn: 0,
            warped_turn: 0,
            // CR 400.7: unearth status is not preserved across zone changes.
            was_unearthed: false,
            // CR 400.7: myriad token exile flag is not preserved across zone changes.
            myriad_exile_at_eoc: false,
            // CR 400.7: decayed sacrifice flag is not preserved across zone changes.
            decayed_sacrifice_at_eoc: false,
            // CR 400.7: ring block sacrifice flag is not preserved across zone changes.
            ring_block_sacrifice_at_eoc: false,
            // CR 400.7: hideaway exile link is cleared on zone change.
            exiled_by_hideaway: None,
            // CR 400.7: encore sacrifice flag is not preserved across zone changes.
            encore_sacrifice_at_end_step: false,
            // CR 400.7: encore mandatory attack target is not preserved across zone changes.
            encore_must_attack: None,
            // CR 400.7: encore original activator is not preserved across zone changes.
            encore_activated_by: None,
            // CR 400.7: delayed end-step sacrifice/exile flags not preserved across zone changes.
            sacrifice_at_end_step: false,
            exile_at_end_step: false,
            return_to_hand_at_end_step: false,
            // CR 400.7: plot status is not preserved across zone changes.
            is_plotted: false,
            plotted_turn: 0,
            is_prototyped: false,
            // CR 400.7: bargained status is not preserved across zone changes.
            was_bargained: false,
            // CR 400.7: collect evidence status is not preserved across zone changes.
            evidence_collected: false,
            // CR 400.7: phasing flags are not preserved across zone changes.
            phased_out_indirectly: false,
            phased_out_controller: None,
            // CR 400.7: devour count is not preserved across zone changes.
            creatures_devoured: 0,
            // CR 702.72a / CR 603.10a: champion_exiled_card is preserved across zone changes
            // so the LTB trigger can read it from the post-move object (last-known information).
            champion_exiled_card: old_object.champion_exiled_card,
            // CR 702.95e / CR 400.7: soulbond pairing is broken on zone change (new object identity).
            paired_with: None,
            // CR 400.7: tribute_was_paid is not preserved across zone changes.
            tribute_was_paid: false,
            // CR 107.3m / CR 400.7: x_value is not preserved across zone changes.
            x_value: 0,
            // CR 702.157a / CR 400.7: squad_count is not preserved across zone changes.
            squad_count: 0,
            // CR 702.175a / CR 400.7: offspring_paid is not preserved across zone changes.
            offspring_paid: false,
            // CR 702.174a / CR 400.7: gift status is not preserved across zone changes.
            gift_was_given: false,
            gift_opponent: None,
            // CR 702.99b / CR 400.7: encoded cipher cards are cleared on zone change.
            // The exiled cards remain in exile but are no longer encoded on anything.
            // CR 702.99c: encoding is broken when the creature leaves the battlefield.
            encoded_cards: imbl::Vector::new(),
            // CR 702.55b / CR 400.7: haunting relationship is cleared on zone change.
            // The exiled haunt card's haunting_target is set AFTER zone move, not inherited.
            haunting_target: None,
            // CR 729.2 / CR 400.7: merged_components are cleared on zone change.
            // When a merged permanent leaves the battlefield, components are split into
            // separate GameObjects (CR 729.3). Each new object starts with empty merged_components.
            merged_components: imbl::Vector::new(),
            // CR 712.8a / CR 400.7: DFC transform state is reset on zone change.
            // The front face is used in all non-battlefield zones (CR 712.8a).
            is_transformed: false,
            last_transform_timestamp: 0,
            // CR 702.145 / CR 400.7: disturb cast status is reset on zone change.
            was_cast_disturbed: false,
            was_cast: false,
            abilities_activated_this_turn: 0,
            // CR 702.167c / CR 400.7: craft exiled materials are cleared on zone change.
            craft_exiled_cards: imbl::Vector::new(),
            // CR 708.2 / CR 400.7: face-down status is cleared on zone change.
            // A face-down permanent leaving the battlefield is revealed (CR 708.9),
            // and the new object in the destination zone is no longer face-down.
            chosen_creature_type: None,
            chosen_color: None,
            face_down_as: None,
            loyalty_ability_activated_this_turn: false,
            class_level: 0,
            designations: Designations::default(),
            // CR 712.4a / CR 400.7: meld component is cleared on zone change.
            adventure_exiled_by: None,
            meld_component: None,

            skip_untap_steps: 0,
        };
        // CR 702.95e: If the departing object was paired, clear the partner's paired_with.
        // We already have old_object.paired_with from the clone taken before removal.
        if let Some(partner_id) = old_object.paired_with {
            if let Some(partner) = self.objects.get_mut(&partner_id) {
                partner.paired_with = None;
            }
        }
        // CR 718.4: When a prototyped permanent leaves the battlefield to any zone
        // that is not the stack or battlefield, revert characteristics to the card's
        // printed values. The prototype-modified P/T, mana_cost, and colors were written
        // into the base characteristics at cast time and must be undone here.
        if old_object.is_prototyped && to != ZoneId::Battlefield && to != ZoneId::Stack {
            if let Some(ref cid) = new_object.card_id {
                if let Some(def) = self.card_registry.get(cid.clone()) {
                    new_object.characteristics.power = def.power;
                    new_object.characteristics.toughness = def.toughness;
                    new_object.characteristics.mana_cost = def.mana_cost.clone();
                    // CR 105.2: colors are derived from the printed mana cost.
                    new_object.characteristics.colors = if let Some(ref mc) = def.mana_cost {
                        crate::rules::casting::colors_from_mana_cost(mc)
                    } else {
                        imbl::OrdSet::new()
                    };
                }
            }
        }
        // CR 729.3: When a merged permanent leaves the battlefield, the primary new object
        // takes the characteristics of the topmost component (merged_components[0]).
        // Without this override, new_object would have the underlying game-object's
        // characteristics (the target permanent's base), not the topmost component's.
        if old_object.zone == ZoneId::Battlefield && !old_object.merged_components.is_empty() {
            let top = &old_object.merged_components[0];
            new_object.characteristics = top.characteristics.clone();
            new_object.card_id = top.card_id.clone();
            new_object.is_token = top.is_token;
        }
        // Add to new zone — MR-M1-02/MR-M1-04: single access, no redundant guard.
        let to_zone = self
            .zones
            .get_mut(&to)
            .ok_or(GameStateError::ZoneNotFound(to))?;
        to_zone.insert(new_id);
        // Insert new object
        self.objects.insert(new_id, new_object);
        // CR 729.3: Merged permanent zone-change splitting.
        // When a merged permanent leaves the battlefield, each component becomes a separate
        // object in the destination zone. The topmost component (index 0) is the primary new
        // object (already created above as `new_id`). Components at indices 1..N get fresh
        // GameObjects created here.
        //
        // CR 729.3a: For graveyard/library, the player may arrange order — we use
        // component order (topmost to bottommost) as the deterministic default.
        // CR 400.7: Each component object starts with empty merged_components (it's a new object).
        // CR 729.2d: Token status is determined per component's `is_token` field.
        if old_object.zone == ZoneId::Battlefield && old_object.merged_components.len() > 1 {
            // Components at indices 1..N become additional objects in `to`.
            let additional_components: Vec<_> = old_object
                .merged_components
                .iter()
                .skip(1)
                .cloned()
                .collect();
            for component in additional_components {
                // Skip tokens — they cease to exist when they leave the battlefield (CR 111.7).
                if component.is_token {
                    continue;
                }
                let component_id = self.next_object_id();
                self.timestamp_counter += 1;
                let component_obj = GameObject {
                    triggered_abilities_fired_this_turn: imbl::OrdSet::new(),
                    id: component_id,
                    card_id: component.card_id,
                    characteristics: component.characteristics,
                    controller: old_object.owner, // Reset controller to owner (CR 400.7)
                    owner: old_object.owner,
                    zone: to,
                    status: crate::state::game_object::ObjectStatus::default(),
                    counters: OrdMap::new(),
                    attachments: Vector::new(),
                    attached_to: None,
                    damage_marked: 0,
                    deathtouch_damage: false,
                    is_token: false, // Non-tokens only (tokens were filtered above)
                    is_emblem: false,
                    timestamp: self.timestamp_counter,
                    has_summoning_sickness: to == ZoneId::Battlefield,
                    entered_turn: if to == ZoneId::Battlefield {
                        Some(self.turn.turn_number)
                    } else {
                        None
                    },
                    goaded_by: imbl::Vector::new(),
                    kicker_times_paid: 0,
                    cast_alt_cost: None,
                    foretold_turn: 0,
                    warped_turn: 0,
                    was_unearthed: false,
                    myriad_exile_at_eoc: false,
                    decayed_sacrifice_at_eoc: false,
                    ring_block_sacrifice_at_eoc: false,
                    exiled_by_hideaway: None,
                    encore_sacrifice_at_end_step: false,
                    encore_must_attack: None,
                    encore_activated_by: None,
                    sacrifice_at_end_step: false,
                    exile_at_end_step: false,
                    return_to_hand_at_end_step: false,
                    is_plotted: false,
                    plotted_turn: 0,
                    is_prototyped: false,
                    was_bargained: false,
                    evidence_collected: false,
                    phased_out_indirectly: false,
                    phased_out_controller: None,
                    creatures_devoured: 0,
                    champion_exiled_card: None,
                    paired_with: None,
                    tribute_was_paid: false,
                    x_value: 0,
                    squad_count: 0,
                    offspring_paid: false,
                    gift_was_given: false,
                    gift_opponent: None,
                    encoded_cards: imbl::Vector::new(),
                    haunting_target: None,
                    // CR 729.3 / CR 400.7: Each split component starts with empty merged_components.
                    merged_components: imbl::Vector::new(),
                    // CR 712.8a / CR 400.7: DFC transform state is reset on zone change.
                    is_transformed: false,
                    last_transform_timestamp: 0,
                    was_cast_disturbed: false,
                    was_cast: false,
                    abilities_activated_this_turn: 0,
                    craft_exiled_cards: imbl::Vector::new(),
                    // CR 708.2 / CR 400.7: face-down status is cleared on zone change.
                    chosen_creature_type: None,
                    chosen_color: None,
                    face_down_as: None,
                    loyalty_ability_activated_this_turn: false,
                    class_level: 0,
                    designations: Designations::default(),
                    // CR 712.4a / CR 400.7: meld pairing is cleared on zone change.
                    adventure_exiled_by: None,
                    meld_component: None,

                    skip_untap_steps: 0,
                };
                // Add component to destination zone and objects map.
                if let Some(zone_set) = self.zones.get_mut(&to) {
                    zone_set.insert(component_id);
                }
                self.objects.insert(component_id, component_obj);
                // CR 702.69a: Track non-topmost components entering graveyard from battlefield.
                if let ZoneId::Graveyard(_) = to {
                    self.permanents_put_into_graveyard_this_turn += 1;
                }
            }
        }
        // CR 712.4a: Meld zone-change splitting.
        // When a melded permanent leaves the battlefield, the meld component card
        // becomes a separate object in the destination zone (similar to Mutate splitting).
        // The primary object (already created as `new_id`) keeps this card's identity.
        // The meld_component card gets a fresh GameObject.
        if old_object.zone == ZoneId::Battlefield {
            if let Some(ref component_card_id) = old_object.meld_component {
                // Clone component characteristics from card registry before mutable borrows.
                let component_data = self
                    .card_registry
                    .get(component_card_id.clone())
                    .map(|def| {
                        let colors = if let Some(ref mc) = def.mana_cost {
                            crate::rules::casting::colors_from_mana_cost(mc)
                        } else {
                            imbl::OrdSet::new()
                        };
                        Characteristics {
                            name: def.name.clone(),
                            mana_cost: def.mana_cost.clone(),
                            card_types: def.types.card_types.clone(),
                            subtypes: def.types.subtypes.clone(),
                            supertypes: def.types.supertypes.clone(),
                            power: def.power,
                            toughness: def.toughness,
                            colors,
                            ..Default::default()
                        }
                    });
                if let Some(component_chars) = component_data {
                    let component_id = self.next_object_id();
                    self.timestamp_counter += 1;
                    let component_obj = GameObject {
                        triggered_abilities_fired_this_turn: imbl::OrdSet::new(),
                        id: component_id,
                        card_id: Some(component_card_id.clone()),
                        characteristics: component_chars,
                        controller: old_object.owner,
                        owner: old_object.owner,
                        zone: to,
                        status: crate::state::game_object::ObjectStatus::default(),
                        counters: OrdMap::new(),
                        attachments: Vector::new(),
                        attached_to: None,
                        damage_marked: 0,
                        deathtouch_damage: false,
                        is_token: false,
                        is_emblem: false,
                        timestamp: self.timestamp_counter,
                        has_summoning_sickness: to == ZoneId::Battlefield,
                        entered_turn: if to == ZoneId::Battlefield {
                            Some(self.turn.turn_number)
                        } else {
                            None
                        },
                        goaded_by: imbl::Vector::new(),
                        kicker_times_paid: 0,
                        cast_alt_cost: None,
                        foretold_turn: 0,
                        warped_turn: 0,
                        was_unearthed: false,
                        myriad_exile_at_eoc: false,
                        decayed_sacrifice_at_eoc: false,
                        ring_block_sacrifice_at_eoc: false,
                        exiled_by_hideaway: None,
                        encore_sacrifice_at_end_step: false,
                        encore_must_attack: None,
                        encore_activated_by: None,
                        sacrifice_at_end_step: false,
                        exile_at_end_step: false,
                        return_to_hand_at_end_step: false,
                        is_plotted: false,
                        plotted_turn: 0,
                        is_prototyped: false,
                        was_bargained: false,
                        evidence_collected: false,
                        phased_out_indirectly: false,
                        phased_out_controller: None,
                        creatures_devoured: 0,
                        champion_exiled_card: None,
                        paired_with: None,
                        tribute_was_paid: false,
                        x_value: 0,
                        squad_count: 0,
                        offspring_paid: false,
                        gift_was_given: false,
                        gift_opponent: None,
                        encoded_cards: imbl::Vector::new(),
                        haunting_target: None,
                        merged_components: imbl::Vector::new(),
                        is_transformed: false,
                        last_transform_timestamp: 0,
                        was_cast_disturbed: false,
                        was_cast: false,
                        abilities_activated_this_turn: 0,
                        craft_exiled_cards: imbl::Vector::new(),
                        chosen_creature_type: None,
                        chosen_color: None,
                        face_down_as: None,
                        loyalty_ability_activated_this_turn: false,
                        class_level: 0,
                        designations: Designations::default(),
                        adventure_exiled_by: None,
                        meld_component: None,

                        skip_untap_steps: 0,
                    };
                    if let Some(zone_set) = self.zones.get_mut(&to) {
                        zone_set.insert(component_id);
                    }
                    self.objects.insert(component_id, component_obj);
                    // CR 702.69a: Track meld component entering graveyard from battlefield.
                    if let ZoneId::Graveyard(_) = to {
                        self.permanents_put_into_graveyard_this_turn += 1;
                    }
                }
            }
        }
        // CR 702.69a: Track permanents entering a graveyard from the battlefield.
        // Tokens count (CR 704.5d — they briefly exist in the graveyard before SBA removes them).
        // Non-permanent cards (instants, sorceries) go from the stack, not the battlefield, so
        // they are naturally excluded by the ZoneId::Battlefield source check.
        if old_object.zone == ZoneId::Battlefield {
            if let ZoneId::Graveyard(_) = to {
                self.permanents_put_into_graveyard_this_turn += 1;
            }
        }
        // MR-M8-16: GC stale `WhileSourceOnBattlefield` replacement effects.
        // When a permanent leaves the battlefield its `ObjectId` is retired (CR 400.7) —
        // any replacement effect sourced on it can never be active again, since the
        // source can never reappear on the battlefield under that same id. `is_effect_active`
        // already returns false for these, so this is purely housekeeping: without it the
        // `replacement_effects` vector grows unbounded over a long game. Targeted removal
        // (only the just-departed object's effects) keeps the cost O(replacement_effects).
        if old_object.zone == ZoneId::Battlefield && to != ZoneId::Battlefield {
            use crate::state::continuous_effect::EffectDuration;
            self.replacement_effects.retain(|e| {
                !(e.duration == EffectDuration::WhileSourceOnBattlefield
                    && e.source == Some(object_id))
            });
        }
        Ok((new_id, old_object))
    }
    /// CR 708.9: Build a `FaceDownRevealed` event for a face-down permanent that is
    /// about to leave the battlefield, if applicable.
    ///
    /// Call this BEFORE `move_object_to_zone` to capture the object's current state.
    /// Returns `Some(event)` only when:
    ///   - The object is on the battlefield
    ///   - `status.face_down == true` and `face_down_as.is_some()`
    ///   - The card's real name is retrievable from the card registry
    ///
    /// The event is used by the M10 network layer to broadcast the card's true identity
    /// to all players when a face-down permanent leaves the battlefield.
    pub fn face_down_reveal_for(
        &self,
        object_id: crate::state::game_object::ObjectId,
    ) -> Option<crate::rules::events::GameEvent> {
        let obj = self.objects.get(&object_id)?;
        if obj.zone != crate::state::zone::ZoneId::Battlefield {
            return None;
        }
        if !obj.status.face_down || obj.face_down_as.is_none() {
            return None;
        }
        let card_id = obj.card_id.as_ref()?;
        let card_name = self
            .card_registry
            .get(card_id.clone())
            .map(|def| def.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        Some(crate::rules::events::GameEvent::FaceDownRevealed {
            player: obj.controller,
            permanent: object_id,
            card_name,
        })
    }
    /// Move an object to the bottom of an ordered zone (CR 702.85a).
    ///
    /// Identical to `move_object_to_zone` but inserts the new object at
    /// position 0 (the bottom) instead of appending to the back (the top).
    /// For unordered zones, behaves the same as `move_object_to_zone`.
    pub(crate) fn move_object_to_bottom_of_zone(
        &mut self,
        object_id: ObjectId,
        to: ZoneId,
    ) -> Result<(ObjectId, GameObject), GameStateError> {
        // Validate destination zone before mutating.
        if !self.zones.contains_key(&to) {
            return Err(GameStateError::ZoneNotFound(to));
        }
        let old_object = self
            .objects
            .get(&object_id)
            .ok_or(GameStateError::ObjectNotFound(object_id))?
            .clone();
        let from = old_object.zone;
        // SR-23: all error checks before capture, so an errored move leaves no ghost
        // snapshot (see `move_object_to_zone`). Object stays present at capture time, so
        // characteristics and the success-path hash are unchanged.
        let from_zone = self
            .zones
            .get(&from)
            .ok_or(GameStateError::ZoneNotFound(from))?;
        if !from_zone.contains(&object_id) {
            return Err(GameStateError::ObjectNotInZone(object_id, from));
        }
        // SR-13: snapshot last-known information before removal (CR 113.7a / 608.2h).
        self.capture_lki_snapshot(object_id, from, &old_object);
        // Remove from old zone. Membership verified above; cannot fail.
        let removed = self
            .zones
            .get_mut(&from)
            .expect("source zone existence checked above")
            .remove(&object_id);
        debug_assert!(removed, "membership checked above, cannot fail");
        // Remove old object from objects map.
        self.objects.remove(&object_id);
        // Create new object with fresh ID (CR 400.7).
        let new_id = self.next_object_id();
        let mut new_object = GameObject {
            triggered_abilities_fired_this_turn: imbl::OrdSet::new(),
            id: new_id,
            card_id: old_object.card_id.clone(),
            characteristics: old_object.characteristics.clone(),
            controller: old_object.owner,
            owner: old_object.owner,
            zone: to,
            status: ObjectStatus::default(),
            counters: OrdMap::new(),
            attachments: Vector::new(),
            attached_to: None,
            damage_marked: 0,
            deathtouch_damage: false,
            is_token: old_object.is_token,
            is_emblem: old_object.is_emblem,
            timestamp: self.timestamp_counter,
            has_summoning_sickness: to == ZoneId::Battlefield,
            entered_turn: if to == ZoneId::Battlefield {
                Some(self.turn.turn_number)
            } else {
                None
            },
            goaded_by: imbl::Vector::new(),
            // CR 400.7: kicked status is not preserved across zone changes.
            kicker_times_paid: 0,
            // CR 400.7: alt-cost status (evoke/escape/dash) is not preserved across zone changes.
            cast_alt_cost: None,
            foretold_turn: 0,
            warped_turn: 0,
            // CR 400.7: unearth status is not preserved across zone changes.
            was_unearthed: false,
            // CR 400.7: myriad token exile flag is not preserved across zone changes.
            myriad_exile_at_eoc: false,
            // CR 400.7: decayed sacrifice flag is not preserved across zone changes.
            decayed_sacrifice_at_eoc: false,
            // CR 400.7: ring block sacrifice flag is not preserved across zone changes.
            ring_block_sacrifice_at_eoc: false,
            // CR 400.7: hideaway exile link is cleared on zone change.
            exiled_by_hideaway: None,
            // CR 400.7: encore sacrifice flag is not preserved across zone changes.
            encore_sacrifice_at_end_step: false,
            // CR 400.7: encore mandatory attack target is not preserved across zone changes.
            encore_must_attack: None,
            // CR 400.7: encore original activator is not preserved across zone changes.
            encore_activated_by: None,
            // CR 400.7: delayed end-step sacrifice/exile flags not preserved across zone changes.
            sacrifice_at_end_step: false,
            exile_at_end_step: false,
            return_to_hand_at_end_step: false,
            // CR 400.7: plot status is not preserved across zone changes.
            is_plotted: false,
            plotted_turn: 0,
            is_prototyped: false,
            // CR 400.7: bargained status is not preserved across zone changes.
            was_bargained: false,
            // CR 400.7: collect evidence status is not preserved across zone changes.
            evidence_collected: false,
            // CR 400.7: phasing flags are not preserved across zone changes.
            phased_out_indirectly: false,
            phased_out_controller: None,
            // CR 400.7: devour count is not preserved across zone changes.
            creatures_devoured: 0,
            // CR 702.72a / CR 603.10a: champion_exiled_card is preserved across zone changes
            // so the LTB trigger can read it from the post-move object (last-known information).
            champion_exiled_card: old_object.champion_exiled_card,
            // CR 702.95e / CR 400.7: soulbond pairing is broken on zone change (new object identity).
            paired_with: None,
            // CR 400.7: tribute_was_paid is not preserved across zone changes.
            tribute_was_paid: false,
            // CR 107.3m / CR 400.7: x_value is not preserved across zone changes.
            x_value: 0,
            // CR 702.157a / CR 400.7: squad_count is not preserved across zone changes.
            squad_count: 0,
            // CR 702.175a / CR 400.7: offspring_paid is not preserved across zone changes.
            offspring_paid: false,
            // CR 702.174a / CR 400.7: gift status is not preserved across zone changes.
            gift_was_given: false,
            gift_opponent: None,
            // CR 702.99b / CR 400.7: encoded cipher cards are cleared on zone change.
            // The exiled cards remain in exile but are no longer encoded on anything.
            // CR 702.99c: encoding is broken when the creature leaves the battlefield.
            encoded_cards: imbl::Vector::new(),
            // CR 702.55b / CR 400.7: haunting relationship is cleared on zone change.
            // The exiled haunt card's haunting_target is set AFTER zone move, not inherited.
            haunting_target: None,
            // CR 729.2 / CR 400.7: merged_components are cleared on zone change.
            // When a merged permanent leaves the battlefield, components are split into
            // separate GameObjects (CR 729.3). Each new object starts with empty merged_components.
            merged_components: imbl::Vector::new(),
            // CR 712.8a / CR 400.7: DFC transform state is reset on zone change.
            // The front face is used in all non-battlefield zones (CR 712.8a).
            is_transformed: false,
            last_transform_timestamp: 0,
            // CR 702.145 / CR 400.7: disturb cast status is reset on zone change.
            was_cast_disturbed: false,
            was_cast: false,
            abilities_activated_this_turn: 0,
            // CR 702.167c / CR 400.7: craft exiled materials are cleared on zone change.
            craft_exiled_cards: imbl::Vector::new(),
            // CR 708.2 / CR 400.7: face-down status is cleared on zone change.
            chosen_creature_type: None,
            chosen_color: None,
            face_down_as: None,
            loyalty_ability_activated_this_turn: false,
            class_level: 0,
            designations: Designations::default(),
            // CR 712.4a / CR 400.7: meld component is cleared on zone change.
            adventure_exiled_by: None,
            meld_component: None,

            skip_untap_steps: 0,
        };
        // CR 702.95e: If the departing object was paired, clear the partner's paired_with.
        if let Some(partner_id) = old_object.paired_with {
            if let Some(partner) = self.objects.get_mut(&partner_id) {
                partner.paired_with = None;
            }
        }
        // CR 718.4: When a prototyped permanent leaves the battlefield to any zone
        // that is not the stack or battlefield, revert characteristics to the card's
        // printed values.
        if old_object.is_prototyped && to != ZoneId::Battlefield && to != ZoneId::Stack {
            if let Some(ref cid) = new_object.card_id {
                if let Some(def) = self.card_registry.get(cid.clone()) {
                    new_object.characteristics.power = def.power;
                    new_object.characteristics.toughness = def.toughness;
                    new_object.characteristics.mana_cost = def.mana_cost.clone();
                    // CR 105.2: colors are derived from the printed mana cost.
                    new_object.characteristics.colors = if let Some(ref mc) = def.mana_cost {
                        crate::rules::casting::colors_from_mana_cost(mc)
                    } else {
                        imbl::OrdSet::new()
                    };
                }
            }
        }
        // Insert at the front (= bottom) of the destination zone.
        let to_zone = self
            .zones
            .get_mut(&to)
            .ok_or(GameStateError::ZoneNotFound(to))?;
        to_zone.push_front(new_id);
        // Insert new object.
        self.objects.insert(new_id, new_object);
        // CR 702.69a: Track permanents entering a graveyard from the battlefield.
        if old_object.zone == ZoneId::Battlefield {
            if let ZoneId::Graveyard(_) = to {
                self.permanents_put_into_graveyard_this_turn += 1;
            }
        }
        Ok((new_id, old_object))
    }
    /// Returns all active (non-lost, non-conceded) player IDs in turn order.
    pub fn active_players(&self) -> Vec<PlayerId> {
        self.turn
            .turn_order
            .iter()
            .filter(|id| {
                self.players
                    .get(id)
                    .map(|p| !p.has_lost && !p.has_conceded)
                    .unwrap_or(false)
            })
            .copied()
            .collect()
    }
    /// Returns the total number of game objects across all zones.
    pub fn total_objects(&self) -> usize {
        self.objects.len()
    }
    /// Find all objects in a given zone.
    pub fn objects_in_zone(&self, zone_id: &ZoneId) -> Vec<&GameObject> {
        match self.zones.get(zone_id) {
            Some(zone) => zone
                .object_ids()
                .iter()
                .filter_map(|id| self.objects.get(id))
                .collect(),
            None => Vec::new(),
        }
    }
}

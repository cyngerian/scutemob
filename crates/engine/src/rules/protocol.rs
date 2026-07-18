//! Wire protocol versioning for `Command` and `GameEvent` streams.
//!
//! `Command` is the only way into the engine and `GameEvent` is the only way out
//! (invariants #3 and #4), so the two enums *are* the wire protocol for M10's
//! centralized server, and they are what a replay log is made of. This module
//! puts a version tag on those serialized streams and defines what happens when
//! it does not match.
//!
//! # Policy: strict lockstep
//!
//! A message declares `protocol_version`. A receiver accepts it **iff** the
//! declared version equals [`PROTOCOL_VERSION`] exactly. Anything else is
//! rejected with [`ProtocolError::VersionMismatch`]. There is no negotiation, no
//! forward compatibility, and no best-effort decoding of an unknown version.
//!
//! The reason is invariant #9. Rewind, replay, and pause all rest on a complete
//! and accurate state history from turn 1. A client that silently drops an event
//! variant it does not understand, or that fills a missing field with a default,
//! holds a history that cannot be correctly rewound — and it holds it *without
//! knowing*. Refusing the connection is recoverable; a corrupted history is not.
//!
//! # The version number is machine-checked, not remembered
//!
//! A hand-bumped constant next to a growing enum is precisely the kind of
//! process guarantee the SR track exists to convert into a machine guarantee: it
//! is correct exactly as long as every future author remembers it. So
//! [`PROTOCOL_SCHEMA_FINGERPRINT`] pins a digest of the **transitive type
//! closure** of the three wire frames — `Command`, `GameEvent`, [`ReplayLog`] —
//! computed from workspace source by `tests/protocol_schema.rs`. Change the
//! shape of anything on the wire and that test fails, names the drift, and tells
//! you to bump [`PROTOCOL_VERSION`].
//!
//! The closure is 90 types, not 3. `GameEvent::CreatureDied` carries
//! `Option<Characteristics>`, which reaches `AbilityInstance` → `Effect` →
//! `TargetFilter` → the whole card DSL. **Adding an `Effect` variant is a wire
//! change**, so most primitive batches (PB-*) will bump this version. That is
//! not gate noise; it is what strict lockstep means.
//!
//! It bottoms out cleanly: `GameState` is *not* in the closure. Whole-state sync
//! is a different question, guarded by `HASH_SCHEMA_VERSION`
//! (`state::hash`), and a replay log carries both — see [`ReplayLog`].
//!
//! Full rationale, bump procedure, and known holes:
//! `docs/mtg-engine-protocol-versioning.md`.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::rules::command::Command;
use crate::state::hash::HASH_SCHEMA_VERSION;

/// The wire protocol version spoken by this build.
///
/// Bump this whenever the serialized shape of `Command`, `GameEvent`, or any
/// type reachable from them changes. `tests/protocol_schema.rs` fails until you
/// do, and tells you which types moved.
///
/// # History
/// - 1: SR-8 (2026-07-10) — initial versioned envelope. Baseline shape is the
///   90-type closure recorded in [`PROTOCOL_SCHEMA_FINGERPRINT`].
/// - 2: SR-10 (2026-07-10) — `Command::CastSpell`'s ~16-field payload boxed into
///   a new [`crate::rules::command::CastSpellData`] struct (clippy::large_enum_variant;
///   shrinks every `Command` value and replay-log entry). The serialized **bytes**
///   are unchanged — a boxed newtype variant wrapping a struct is serde-identical to
///   the former struct variant — but the shape digest moved because the closure grew
///   by one type (90 → 91) and the variant's declared form changed. Bumped per this
///   gate's policy that any non-variant-reorder digest move bumps the version.
/// - 3: SR-34 (2026-07-17) — `ManaAbility` (reachable from `Command`/`GameEvent` via
///   `Characteristics.mana_abilities: Vec<ManaAbility>`, a [`CLOSURE_MUST_CONTAIN`]
///   entry) gains `mana_cost: Option<ManaCost>` and `life_cost: u32`, its activation
///   cost's mana and life components (CR 605.1a — a mana ability is classified by
///   what it does, not what it costs; `handle_tap_for_mana` now pays these). The
///   closure stays 91 types (no new type joins it, `ManaCost` was already in the
///   closure), but `ManaAbility`'s declared shape changed, so the digest moves.
/// - 4: SR-36 (2026-07-17) — SF-8/SF-9: `ManaAbility` gains
///   `scaled_amount: Option<Box<EffectAmount>>` (a dynamic mana amount, CR 605.1a) and
///   `ActivationCost` (reachable via `Characteristics.activated_abilities: Vec<ActivatedAbility>`
///   → `ActivatedAbility.cost: ActivationCost`) gains `life_cost: u32` (CR 118.3/119.4
///   — a non-mana activated ability's life-payment component). `EffectAmount` was
///   already in the closure (via `Effect`), so the closure's type count is unchanged;
///   both structs' declared shapes moved, so the digest moves.
/// - 5: SR-37 (2026-07-17) — SF-10: `ManaAbility` gains
///   `activation_condition: Option<Condition>` (an "activate only if ..." restriction,
///   CR 605.1a + CR 602.5b — Tainted Field's coloured arms). `Condition` was already in
///   the closure (reachable via `Effect::Conditional`), so the closure's type count is
///   unchanged; `ManaAbility`'s declared shape moved, so the digest moves.
/// - 6: PB-EF1 (2026-07-18) — `ActivationCost` (reachable via
///   `Characteristics.activated_abilities` → `ActivatedAbility.cost`) gains
///   `sacrifice_exclude_self: bool` (CR 109.1 — the "Sacrifice ANOTHER [permanent]"
///   restriction on an activated ability's sacrifice cost; `SacrificeFilter` carries no
///   ObjectId so the bit rides on `ActivationCost`). The closure's type count is
///   unchanged; `ActivationCost`'s declared shape moved, so the digest moves.
/// - 7: PB-EF2 (2026-07-18) — `TokenSpec` (reachable via `Effect::CreateToken`/
///   `Effect::CreateTokenAndAttachSource`) gains `recipient: PlayerTarget` (CR 111.1 /
///   CR 608.2h — which player creates the token(s); "its controller creates …" cards
///   like Swan Song). `PlayerTarget` (already in the closure) gains two variants,
///   `ControllerOfCounteredSpell` and `ControllerOfTriggeringObject`. The closure's
///   type count is unchanged; both types' declared shapes moved, so the digest moves.
/// - 8: PB-EF3 (2026-07-18) — `EffectTarget` (reachable via `Effect::DealDamage.target`
///   and other `Effect` variants) gains `AttackTarget`; `PlayerTarget` (already in the
///   closure) gains `DefendingPlayer` (CR 508.4 — the defending player / attack target
///   of an attacking creature, EF-W-MISS-4/EF-W-MISS-10). The closure's type count is
///   unchanged; both types' declared shapes moved, so the digest moves.
/// - 9: PB-EF4 (2026-07-18) — `Effect::DealDamage` (reachable via `Effect` and thus in
///   the closure already) gains `source: Option<EffectTarget>` (CR 119.3 / 702.15a — an
///   optional damage-source override, e.g. "the entering creature deals it", resolved
///   to a single ObjectId at execution time; `EffectTarget` was already in the closure).
///   The closure's type count is unchanged; `Effect`'s declared shape moved, so the
///   digest moves. (`EffectFilter` also gained a `TriggeringCreature` variant in this
///   PB, but `EffectFilter` is off the wire closure — it lives inside `GameState`'s
///   `continuous_effects`, not `Command`/`GameEvent` — so that half is a HASH_SCHEMA_VERSION
///   bump only, not a PROTOCOL_VERSION one.)
/// - 10: PB-EF5 (2026-07-18) — `Effect` (already in the closure) gains a new unit
///   variant `TransformSelf` (CR 701.27a/f, 712.18 — flip the resolving ability's own
///   source DFC in place; used by an on-card triggered/activated/conditional effect,
///   distinct from the existing `Command::Transform`). The closure's type count is
///   unchanged; `Effect`'s declared shape moved, so the digest moves.
/// - 11: PB-EF6 (2026-07-18) — `TargetRequirement` (reachable via
///   `AbilityDefinition.targets` / `Effect`) gains a new unit variant `TargetOpponent`
///   (CR 102.3/102.4/115.1/601.2c/603.3d — "target opponent", an opponent-restricted
///   player target; EF-W-PB2-2). The closure's type count is unchanged;
///   `TargetRequirement`'s declared shape moved, so the digest moves.
/// - 12: PB-EF7 (2026-07-18) — `Command::ActivateAbility` (a wire frame) gains
///   `modes_chosen: Vec<usize>`, and `AbilityDefinition::Activated` (reachable via
///   `Characteristics.activated_abilities` → `ActivatedAbility` → the DSL closure)
///   gains `modes: Option<ModeSelection>` (CR 700.2a/601.2b — modal activated
///   abilities; EF-W-PB2-4). `ModeSelection` was already in the closure (via
///   `AbilityDefinition::Spell`/`Triggered`). The closure's type count is unchanged;
///   both `Command` and `AbilityDefinition`'s declared shapes moved, so the digest moves.
/// - 13: PB-EF8 (2026-07-18) — `Cost` (reachable via `AbilityDefinition::Activated.cost`)
///   gains a new unit variant `ExileSelfFromHand` (CR 118 + CR 400.7 + CR 605.1a — a
///   from-hand mana ability's exile-self activation cost, e.g. Simian/Elvish Spirit
///   Guide), and `ActivationZone` (reachable via `AbilityDefinition::Activated.activation_zone`)
///   gains a new unit variant `Hand` (CR 602.2 — decorative marker; the mana-lowering
///   path keys off `Cost::ExileSelfFromHand` alone, not this field). The closure's type
///   count is unchanged; both `Cost` and `ActivationZone`'s declared shapes moved, so the
///   digest moves.
/// - 14: PB-EF9 (2026-07-18) — `EffectDuration` (reachable via `Effect::GainControl` /
///   `Effect::ApplyContinuousEffect(ContinuousEffectDef)` → the card DSL closure) gains
///   a new variant `WhileYouControlSource(PlayerId)` (CR 611.2b/c — "for as long as you
///   control [source]", a continuous-effect duration for gain-control effects; Olivia
///   Voldaren, Dragonlord Silumgar). The closure's type count is unchanged;
///   `EffectDuration`'s declared shape moved, so the digest moves.
pub const PROTOCOL_VERSION: u32 = 14;

/// Digest of the serialized shape of the wire-frame type closure
/// (`Command`, `GameEvent`, [`ReplayLog`] and everything they reach).
///
/// Recomputed from workspace source by `tests/protocol_schema.rs` and compared
/// against this constant. A mismatch means the wire format changed. Update this
/// value **and** bump [`PROTOCOL_VERSION`] in the same commit.
///
/// The one exception: widening the *definition* of the closure (adding a scan
/// root, a protocol root, or an `EXTERNAL_TYPES` entry) also moves the digest
/// without any wire change. Re-pin without bumping, and say so in the commit.
///
/// This is a shape digest, not a semantic one: renaming a field, adding a
/// variant, or adding `#[serde(skip)]` all move it, but redefining what an
/// existing `u32` *means* does not. Semantic changes still require a manual
/// [`PROTOCOL_VERSION`] bump.
pub const PROTOCOL_SCHEMA_FINGERPRINT: &str =
    "b94f90e1c6d7f4193385489f6f6d541dbb764534eab09593584f99361ea828d7";

/// One `(version, fingerprint)` row of the append-only protocol-schema history.
///
/// The wire-protocol analogue of [`crate::state::hash::HashSchemaEpoch`] (SR-17).
/// The protocol has a single shape digest — [`PROTOCOL_SCHEMA_FINGERPRINT`] — with
/// no separate hash byte-stream, so one fingerprint per row (not two).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtocolEpoch {
    /// The [`PROTOCOL_VERSION`] this row pins.
    pub version: u32,
    /// [`PROTOCOL_SCHEMA_FINGERPRINT`] as of that version — blake3 of the
    /// normalized declaration text of the wire-frame type closure.
    pub fingerprint: &'static str,
}

/// SR-27: append-only ledger backing [`PROTOCOL_VERSION`], mirroring
/// [`crate::state::hash::HASH_SCHEMA_HISTORY`] (SR-17).
///
/// # Why this exists on top of the fingerprint
///
/// [`PROTOCOL_SCHEMA_FINGERPRINT`] makes the version *shape-derived*: you cannot
/// change the wire without `tests/core/protocol_schema.rs` reddening and naming the
/// drift. But it does not stop the *other* half of the cheat — re-pin the
/// fingerprint to the new value and skip the [`PROTOCOL_VERSION`] bump. The
/// recompute gate goes green again and a wire change ships under the old version,
/// so two builds with incompatible shapes both claim the same version and
/// mis-decode each other *silently* — precisely the failure strict lockstep exists
/// to prevent. The `protocol_version_sentinel` forces you to *notice* a bump, never
/// to *make* one.
///
/// This table closes that. It is **append-only**: the tail row is the live schema
/// (validated by recomputation in `protocol_schema.rs`), and every row behind it is
/// shipped-and-superseded and frozen. The test pins the baseline row against its own
/// FROZEN constants and pins a digest of the whole frozen prefix, so re-pinning any
/// row in place — including the current one, while it is still the baseline — fails.
///
/// # Append-only bump procedure
///
/// To change the wire protocol, in one commit:
///   1. bump [`PROTOCOL_VERSION`] and add its `- N:` History line above;
///   2. **append** a new row here whose `fingerprint` is the recomputed digest
///      (read it from the `protocol_schema.rs` failure text) and set
///      [`PROTOCOL_SCHEMA_FINGERPRINT`] to the same value;
///   3. update the `protocol_version_sentinel` and the FROZEN prefix digest in
///      `protocol_schema.rs`.
///
/// Never edit an existing row.
///
/// The baseline is version 2 (the version at SR-27 time). Versions 1..=1 predate
/// this ledger and are not reconstructed — exactly as SR-17 started
/// `HASH_SCHEMA_HISTORY` at the then-current version rather than back-filling.
pub const PROTOCOL_HISTORY: &[ProtocolEpoch] = &[
    ProtocolEpoch {
        version: 2,
        // SR-27 (2026-07-16): baseline. Pins whatever PROTOCOL_VERSION 2 already was
        // (the 91-type closure after SR-10 boxed CastSpell). Same value as
        // PROTOCOL_SCHEMA_FINGERPRINT; the two are kept in lockstep by
        // `history_tail_matches_the_fingerprint_const`.
        fingerprint: "ba7907d9f51a65acba39ccf020a14bd6234f637731c934490a7cbf749e5f97b6",
    },
    ProtocolEpoch {
        version: 3,
        // SR-34 (2026-07-17): ManaAbility gained mana_cost/life_cost (see the `- 3:`
        // History line above).
        fingerprint: "c23d09a7956b239cc1a4edfe629b268b37a2918138def227c9ba373d805ea0f6",
    },
    ProtocolEpoch {
        version: 4,
        // SR-36 (2026-07-17): ManaAbility gained scaled_amount; ActivationCost gained
        // life_cost (see the `- 4:` History line above).
        fingerprint: "45dd82a14adf0b7e2247f7d22fad32c017adf9a25cc4129c92c489513c4ae4d4",
    },
    ProtocolEpoch {
        version: 5,
        // SR-37 (2026-07-17): ManaAbility gained activation_condition (see the `- 5:`
        // History line above).
        fingerprint: "e8d28a23ccc2a1ba7c7b2643b33bb32b0374e0651b3eb6b60ec15f4817e3a85a",
    },
    ProtocolEpoch {
        version: 6,
        // PB-EF1 (2026-07-18): ActivationCost gained sacrifice_exclude_self (see the
        // `- 6:` History line above).
        fingerprint: "df270ca1b58b7fa17bfa2ca56afb564de4f8de22cc15770da511b3a6c7c7a4dc",
    },
    ProtocolEpoch {
        version: 7,
        // PB-EF2 (2026-07-18): TokenSpec gained recipient; PlayerTarget gained
        // ControllerOfCounteredSpell/ControllerOfTriggeringObject (see the `- 7:`
        // History line above).
        fingerprint: "c5931e6163641a6a3f5501a3fc080867a05508047e4c766f2fec415d2b47ef8f",
    },
    ProtocolEpoch {
        version: 8,
        // PB-EF3 (2026-07-18): EffectTarget gained AttackTarget; PlayerTarget gained
        // DefendingPlayer (see the `- 8:` History line above).
        fingerprint: "f5a61a19da2e912416c7bf6ee58acb7cacb0966681868a6810bc8af6d2285ee8",
    },
    ProtocolEpoch {
        version: 9,
        // PB-EF4 (2026-07-18): Effect::DealDamage gained source: Option<EffectTarget>
        // (see the `- 9:` History line above).
        fingerprint: "9bf63ef25ae621acf53155feaa21f01131d35fc7ad6db34b04e35900cb825ac5",
    },
    ProtocolEpoch {
        version: 10,
        // PB-EF5 (2026-07-18): Effect gained TransformSelf (see the `- 10:` History
        // line above).
        fingerprint: "ec3ccb9e5c1cbdc834c86d6fbbc5d8ee6914e1fe1ef44eeee26d078bbea3d618",
    },
    ProtocolEpoch {
        version: 11,
        // PB-EF6 (2026-07-18): TargetRequirement gained TargetOpponent (see the `- 11:`
        // History line above).
        fingerprint: "07e514663c1b64b1831d2aaf0ee95c3e6bf62a3a1ff0b15dd3ca4316a022e739",
    },
    ProtocolEpoch {
        version: 12,
        // PB-EF7 (2026-07-18): Command::ActivateAbility gained modes_chosen;
        // AbilityDefinition::Activated gained modes (see the `- 12:` History line above).
        fingerprint: "05eaa04bf425a625415c58b3f44e6e75489c90deba14a80f7f99c91369a60cde",
    },
    ProtocolEpoch {
        version: 13,
        // PB-EF8 (2026-07-18): Cost gained ExileSelfFromHand; ActivationZone gained
        // Hand (see the `- 13:` History line above).
        fingerprint: "379fb0c4f791138a405b8b47f7efe629c9a870e026db99629da3b709ec83bafa",
    },
    ProtocolEpoch {
        version: 14,
        // PB-EF9 (2026-07-18): EffectDuration gained WhileYouControlSource (see the
        // `- 14:` History line above).
        fingerprint: "b94f90e1c6d7f4193385489f6f6d541dbb764534eab09593584f99361ea828d7",
    },
];

/// Why a versioned message could not be decoded.
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    /// The message declared a protocol version this build does not speak.
    ///
    /// This is the strict-lockstep rejection. The payload is **not** inspected;
    /// under a different version its shape is not knowable.
    #[error(
        "protocol version mismatch: this build speaks v{expected}, message declares v{found}. \
         Strict lockstep — client and server must run the same protocol version."
    )]
    VersionMismatch { expected: u32, found: u32 },

    /// The bytes are not a versioned envelope at all — most often an untagged
    /// message from before versioning existed, or a truncated stream.
    #[error("malformed envelope (no readable `protocol_version` field): {0}")]
    MalformedEnvelope(String),

    /// The version matched but the payload did not decode. This is a genuine
    /// bug (a peer at our own version sent something we cannot read), not a
    /// compatibility problem.
    #[error("payload failed to decode at protocol v{version}: {source}")]
    Payload {
        version: u32,
        #[source]
        source: serde_json::Error,
    },

    /// A replay log was recorded against a different state-hash schema, so its
    /// commands may decode cleanly yet replay to a state whose hash cannot be
    /// compared against the recorded one.
    #[error(
        "replay log state-hash schema mismatch: this build uses HASH_SCHEMA_VERSION {expected}, \
         log records {found}"
    )]
    HashSchemaMismatch { expected: u8, found: u8 },

    /// The value could not be serialized.
    #[error("failed to encode payload: {0}")]
    Encode(#[from] serde_json::Error),
}

/// A payload plus the protocol version that describes its shape.
///
/// `protocol_version` is serialized first and read on its own (see [`decode`]),
/// so a version mismatch is reported as [`ProtocolError::VersionMismatch`]
/// rather than as an opaque serde error about a field the reader has never
/// heard of.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope<T> {
    /// The protocol version the sender was built against.
    pub protocol_version: u32,
    /// The `Command`, `GameEvent`, `ReplayLog`, or batch thereof.
    pub payload: T,
}

impl<T> Envelope<T> {
    /// Wrap a payload at this build's [`PROTOCOL_VERSION`].
    pub fn new(payload: T) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            payload,
        }
    }

    /// Unwrap, discarding the (already-validated) version tag.
    pub fn into_payload(self) -> T {
        self.payload
    }
}

/// Reads only the version tag, ignoring the payload entirely.
///
/// This is the whole reason decoding is staged: the payload's *shape* is only
/// knowable once the version is known to match, so it must not be parsed first.
#[derive(Deserialize)]
struct VersionProbe {
    protocol_version: u32,
}

/// Serialize a payload inside a versioned envelope.
///
/// Works for `Command`, `GameEvent`, `Vec<Command>`, [`ReplayLog`] — anything
/// `Serialize`.
pub fn encode<T: Serialize>(payload: &T) -> Result<String, ProtocolError> {
    Ok(serde_json::to_string(&Envelope::new(payload))?)
}

/// Deserialize a versioned envelope, rejecting any version but our own.
///
/// Staged on purpose:
/// 1. read `protocol_version` alone;
/// 2. reject a mismatch **before** touching the payload;
/// 3. only then decode the payload.
///
/// Step 2 is what makes [`ProtocolError::VersionMismatch`] reachable. Decoding
/// straight into `Envelope<T>` would instead surface an old message as a serde
/// error about an unknown variant — true, but useless to a client deciding
/// whether to reconnect or to tell the user to upgrade.
pub fn decode<T: DeserializeOwned>(json: &str) -> Result<T, ProtocolError> {
    let probe: VersionProbe =
        serde_json::from_str(json).map_err(|e| ProtocolError::MalformedEnvelope(e.to_string()))?;

    if probe.protocol_version != PROTOCOL_VERSION {
        return Err(ProtocolError::VersionMismatch {
            expected: PROTOCOL_VERSION,
            found: probe.protocol_version,
        });
    }

    let envelope: Envelope<T> =
        serde_json::from_str(json).map_err(|source| ProtocolError::Payload {
            version: probe.protocol_version,
            source,
        })?;

    Ok(envelope.payload)
}

/// A recorded command stream: everything needed to replay a game from turn 1.
///
/// Carries **two** versions because a replay must answer two different
/// questions, and passing one does not imply passing the other:
///
/// - `protocol_version` (on the [`Envelope`]) — can this build *read* the
///   commands?
/// - `hash_schema_version` — can this build's state hashes be *compared*
///   against the ones this log was recorded alongside?
///
/// A log can decode perfectly and still replay to states whose hashes are
/// incomparable, which would silently break the desync detection that invariant
/// #9's history rests on. [`decode_replay_log`] checks both.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayLog {
    /// `state::hash::HASH_SCHEMA_VERSION` at record time.
    pub hash_schema_version: u8,
    /// The commands, in submission order.
    pub commands: Vec<Command>,
}

impl ReplayLog {
    /// Record a command stream against this build's state-hash schema.
    pub fn new(commands: Vec<Command>) -> Self {
        Self {
            hash_schema_version: HASH_SCHEMA_VERSION,
            commands,
        }
    }
}

/// Encode a replay log inside a versioned envelope.
pub fn encode_replay_log(log: &ReplayLog) -> Result<String, ProtocolError> {
    encode(log)
}

/// Decode a replay log, checking the protocol version *and* the state-hash schema.
///
/// The hash-schema check is deliberately separate from and after the protocol
/// check: a log whose commands we cannot read is a different failure from one we
/// can read but cannot verify.
pub fn decode_replay_log(json: &str) -> Result<ReplayLog, ProtocolError> {
    let log: ReplayLog = decode(json)?;
    if log.hash_schema_version != HASH_SCHEMA_VERSION {
        return Err(ProtocolError::HashSchemaMismatch {
            expected: HASH_SCHEMA_VERSION,
            found: log.hash_schema_version,
        });
    }
    Ok(log)
}

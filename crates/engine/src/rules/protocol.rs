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
//! closure** of `Command` and `GameEvent`, computed from workspace source by
//! `tests/protocol_schema.rs`. Change the shape of anything on the wire and that
//! test fails, names the drift, and tells you to bump [`PROTOCOL_VERSION`].
//!
//! The closure is 89 types, not 2. `GameEvent::CreatureDied` carries
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
///   88-type closure recorded in [`PROTOCOL_SCHEMA_FINGERPRINT`].
pub const PROTOCOL_VERSION: u32 = 1;

/// Digest of the serialized shape of the `Command` / `GameEvent` type closure.
///
/// Recomputed from workspace source by `tests/protocol_schema.rs` and compared
/// against this constant. A mismatch means the wire format changed. Update this
/// value **and** bump [`PROTOCOL_VERSION`] in the same commit.
///
/// This is a shape digest, not a semantic one: renaming a field, adding a
/// variant, or adding `#[serde(skip)]` all move it, but redefining what an
/// existing `u32` *means* does not. Semantic changes still require a manual
/// [`PROTOCOL_VERSION`] bump.
pub const PROTOCOL_SCHEMA_FINGERPRINT: &str =
    "655184d666e7d2085ca1b0b36c69fe5cea33a877d4155569575e0f5b5eb47748";

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

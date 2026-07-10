//! SR-8: round-trip and mismatch behavior for the versioned wire protocol.
//!
//! Acceptance criterion: a current-version message is accepted; a
//! mismatched-version message is rejected with a **typed** error. The typed part
//! matters — a client that gets `VersionMismatch` can tell its user to upgrade,
//! whereas a client that gets "unknown variant `Foo` at line 1 column 42" can
//! only guess.
//!
//! The load-bearing test here is
//! `version_is_checked_before_the_payload_is_parsed`: it is the one that would
//! fail if someone "simplified" `decode` into a single
//! `serde_json::from_str::<Envelope<T>>`, which still rejects old messages but
//! reports them as payload corruption.

use mtg_engine::{
    decode, decode_replay_log, encode, encode_replay_log, Command, Envelope, GameEvent, ObjectId,
    PlayerId, ProtocolError, ReplayLog, HASH_SCHEMA_VERSION, PROTOCOL_VERSION,
};

fn a_command() -> Command {
    Command::PassPriority {
        player: PlayerId(0),
    }
}

fn another_command() -> Command {
    Command::PlayLand {
        player: PlayerId(1),
        card: ObjectId(42),
    }
}

fn an_event() -> GameEvent {
    GameEvent::TurnStarted {
        player: PlayerId(2),
        turn_number: 7,
    }
}

/// Re-tag an encoded message with an arbitrary version, leaving the payload
/// untouched. This is what a peer running a different build would send.
fn retag(json: &str, version: u32) -> String {
    let mut value: serde_json::Value = serde_json::from_str(json).expect("valid envelope json");
    value["protocol_version"] = serde_json::json!(version);
    value.to_string()
}

// ── Happy path ───────────────────────────────────────────────────────────────

#[test]
fn command_round_trips_at_the_current_version() {
    let wire = encode(&a_command()).expect("encode");
    let decoded: Command = decode(&wire).expect("decode at current version");
    assert_eq!(decoded, a_command());
}

#[test]
fn game_event_round_trips_at_the_current_version() {
    let wire = encode(&an_event()).expect("encode");
    let decoded: GameEvent = decode(&wire).expect("decode at current version");
    assert_eq!(decoded, an_event());
}

/// A batch — what the server broadcasts after one `process_command`.
#[test]
fn event_batch_round_trips() {
    let events = vec![an_event(), GameEvent::AllPlayersPassed];
    let wire = encode(&events).expect("encode");
    let decoded: Vec<GameEvent> = decode(&wire).expect("decode");
    assert_eq!(decoded, events);
}

/// A replay log — the artifact invariant #9's rewind/replay rests on.
#[test]
fn replay_log_round_trips() {
    let log = ReplayLog::new(vec![a_command(), another_command()]);
    let wire = encode_replay_log(&log).expect("encode");
    let decoded = decode_replay_log(&wire).expect("decode");
    assert_eq!(decoded, log);
    assert_eq!(decoded.hash_schema_version, HASH_SCHEMA_VERSION);
    assert_eq!(decoded.commands.len(), 2);
}

/// The version tag is actually on the wire, under a stable field name. A
/// receiver written in another language must be able to find it.
#[test]
fn the_version_tag_is_present_in_the_serialized_bytes() {
    let wire = encode(&a_command()).expect("encode");
    let value: serde_json::Value = serde_json::from_str(&wire).expect("valid json");
    assert_eq!(
        value["protocol_version"],
        serde_json::json!(PROTOCOL_VERSION),
        "serialized envelope must carry `protocol_version`; got {wire}"
    );
    assert!(
        value.get("payload").is_some(),
        "serialized envelope must carry `payload`; got {wire}"
    );
}

#[test]
fn envelope_new_stamps_the_current_version() {
    let envelope = Envelope::new(a_command());
    assert_eq!(envelope.protocol_version, PROTOCOL_VERSION);
    assert_eq!(envelope.into_payload(), a_command());
}

// ── Mismatch handling ────────────────────────────────────────────────────────

#[test]
fn an_older_protocol_version_is_rejected_with_a_typed_error() {
    let wire = retag(&encode(&a_command()).expect("encode"), PROTOCOL_VERSION - 1);
    match decode::<Command>(&wire) {
        Err(ProtocolError::VersionMismatch { expected, found }) => {
            assert_eq!(expected, PROTOCOL_VERSION);
            assert_eq!(found, PROTOCOL_VERSION - 1);
        }
        other => panic!("expected VersionMismatch, got {other:?}"),
    }
}

#[test]
fn a_newer_protocol_version_is_rejected_with_a_typed_error() {
    let wire = retag(&encode(&an_event()).expect("encode"), PROTOCOL_VERSION + 1);
    match decode::<GameEvent>(&wire) {
        Err(ProtocolError::VersionMismatch { expected, found }) => {
            assert_eq!(expected, PROTOCOL_VERSION);
            assert_eq!(found, PROTOCOL_VERSION + 1);
        }
        other => panic!("expected VersionMismatch, got {other:?}"),
    }
}

/// **Strict lockstep has no forward compatibility.** A future message whose
/// payload happens to be readable today is still refused. Accepting it would
/// mean holding a history missing whatever the new version added — the silent
/// corruption invariant #9 forbids.
#[test]
fn a_future_version_is_rejected_even_when_its_payload_would_parse() {
    let wire = retag(&encode(&a_command()).expect("encode"), 9999);
    // The payload is a perfectly valid current-version Command...
    let value: serde_json::Value = serde_json::from_str(&wire).unwrap();
    assert!(serde_json::from_value::<Command>(value["payload"].clone()).is_ok());
    // ...and it is rejected anyway.
    assert!(matches!(
        decode::<Command>(&wire),
        Err(ProtocolError::VersionMismatch { .. })
    ));
}

/// The staging guarantee. An old message carrying a payload this build cannot
/// parse must report *why it is old*, not *how it failed to parse*.
///
/// A single-pass `from_str::<Envelope<T>>` would emit `Payload`/serde noise
/// here. That is the regression this test exists to catch.
#[test]
fn version_is_checked_before_the_payload_is_parsed() {
    let wire = format!(
        r#"{{"protocol_version":{},"payload":{{"SomeVariantDeletedLongAgo":{{"x":1}}}}}}"#,
        PROTOCOL_VERSION - 1
    );
    match decode::<Command>(&wire) {
        Err(ProtocolError::VersionMismatch { expected, found }) => {
            assert_eq!(expected, PROTOCOL_VERSION);
            assert_eq!(found, PROTOCOL_VERSION - 1);
        }
        other => panic!(
            "an unreadable payload at an OLD version must surface as VersionMismatch, not as a \
             payload error — the version check must run first. Got {other:?}"
        ),
    }
}

/// Conversely: at *our* version, an unreadable payload is a genuine bug and must
/// not be disguised as a compatibility problem.
#[test]
fn a_bad_payload_at_the_current_version_is_a_payload_error() {
    let wire =
        format!(r#"{{"protocol_version":{PROTOCOL_VERSION},"payload":{{"NoSuchVariant":{{}}}}}}"#);
    match decode::<Command>(&wire) {
        Err(ProtocolError::Payload { version, .. }) => assert_eq!(version, PROTOCOL_VERSION),
        other => panic!("expected Payload, got {other:?}"),
    }
}

/// An untagged message — e.g. anything serialized before SR-8 existed — is not
/// mistaken for version 0.
#[test]
fn an_untagged_message_is_a_malformed_envelope() {
    let bare = serde_json::to_string(&a_command()).expect("raw command json");
    assert!(
        matches!(
            decode::<Command>(&bare),
            Err(ProtocolError::MalformedEnvelope(_))
        ),
        "a bare, untagged Command must not decode as if it were versioned"
    );
}

#[test]
fn garbage_is_a_malformed_envelope() {
    assert!(matches!(
        decode::<Command>("not json at all"),
        Err(ProtocolError::MalformedEnvelope(_))
    ));
    assert!(matches!(
        decode::<Command>(r#"{"payload":null}"#),
        Err(ProtocolError::MalformedEnvelope(_))
    ));
}

// ── Replay-log compatibility ─────────────────────────────────────────────────

/// A replay log answers two questions. Passing the protocol check does not imply
/// passing the hash-schema check, and conflating them would let a log replay to
/// states whose hashes silently cannot be compared with the recorded ones.
#[test]
fn a_replay_log_from_a_different_hash_schema_is_rejected_separately() {
    let mut log = ReplayLog::new(vec![a_command()]);
    log.hash_schema_version = HASH_SCHEMA_VERSION.wrapping_sub(1);
    let wire = encode_replay_log(&log).expect("encode");

    // It decodes as a protocol message just fine...
    assert!(decode::<ReplayLog>(&wire).is_ok());

    // ...but the replay-log reader refuses it.
    match decode_replay_log(&wire) {
        Err(ProtocolError::HashSchemaMismatch { expected, found }) => {
            assert_eq!(expected, HASH_SCHEMA_VERSION);
            assert_eq!(found, HASH_SCHEMA_VERSION.wrapping_sub(1));
        }
        other => panic!("expected HashSchemaMismatch, got {other:?}"),
    }
}

/// Protocol mismatch is reported before hash-schema mismatch: a log we cannot
/// read is a different failure from one we can read but cannot verify.
#[test]
fn protocol_mismatch_outranks_hash_schema_mismatch() {
    let mut log = ReplayLog::new(vec![a_command()]);
    log.hash_schema_version = HASH_SCHEMA_VERSION.wrapping_sub(1);
    let wire = retag(
        &encode_replay_log(&log).expect("encode"),
        PROTOCOL_VERSION - 1,
    );
    assert!(matches!(
        decode_replay_log(&wire),
        Err(ProtocolError::VersionMismatch { .. })
    ));
}

// ── Error ergonomics ─────────────────────────────────────────────────────────

/// The rejection must tell a human what to do. A client surfaces this string.
#[test]
fn version_mismatch_displays_actionably() {
    let err = ProtocolError::VersionMismatch {
        expected: 3,
        found: 2,
    };
    let msg = err.to_string();
    assert!(msg.contains("v3") && msg.contains("v2"), "got: {msg}");
    assert!(
        msg.contains("same protocol version"),
        "the message must say what the operator has to do; got: {msg}"
    );
}

//! PB-DX56 (`OOS-DX22-8`) — **both** CR 400.7 zone-move helpers must perform the
//! attachment fix-up, and the reverse direction must stay out of them.
//!
//! # Why this is a source gate and not a probe
//!
//! `GameState::move_object_to_bottom_of_zone` is `pub(crate)`, so an integration test —
//! which is an external crate — **cannot call it**. Removing the
//! `detach_from_host_on_departure` call from that site alone was executed as bypass row
//! **D1** and left the whole workspace GREEN, and the honest reading is *"no probe AND no
//! currently reachable production path"* rather than *"an untested defect"*: tracing the
//! callers of the bottom-of-library helper finds none that reaches it with an attached
//! battlefield permanent (the general `Effect::MoveZone`-to-library-bottom does not route
//! through it). That makes a behavioural probe unwritable today and a source gate the
//! right instrument — the property being asserted is a WIRING fact about two siblings that
//! must not drift, which is exactly what `state/mod.rs`'s CR 702.95e soulbond fix-up is
//! already duplicated across.
//!
//! Disclosed here rather than only in `memory/`, per this project's UNDISCRIMINATED-row
//! convention.

use std::path::PathBuf;

fn state_mod_src() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/state/mod.rs");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// The brace-matched body of `fn <name>(`, so a byte window cannot fail OPEN by
/// over-scanning into the next function and vouching for a call that is not there
/// (`OOS-DX49-2`).
fn body_of(src: &str, needle: &str) -> String {
    let at = src
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} must exist in state/mod.rs"));
    let open = at + src[at..].find('{').expect("body opens");
    let bytes = src.as_bytes();
    let mut depth = 0usize;
    for (i, b) in bytes.iter().enumerate().skip(open) {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return src[open..=i].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("{needle}'s body is unbalanced");
}

/// **Bypass D1, closed.** Both zone-move helpers must call the shared CR 400.7 attachment
/// fix-up. Removing it from one leaves that helper silently retiring an attacher's id while
/// its host keeps the dead `ObjectId` forever (`OOS-DX22-8`).
#[test]
fn r1_both_zone_move_helpers_detach_from_the_host() {
    let src = state_mod_src();
    for site in [
        "pub(crate) fn move_object_to_zone(",
        "pub(crate) fn move_object_to_bottom_of_zone(",
    ] {
        let body = body_of(&src, site);
        assert!(
            body.len() > 500,
            "non-vacuity: {site}'s brace-matched body is only {} bytes, which cannot be \
             the real function -- the matcher has gone wrong and every assertion below \
             would be vacuous",
            body.len()
        );
        assert!(
            body.contains("detach_from_host_on_departure("),
            "CR 400.7 (`OOS-DX22-8`): `{site}` retires an object's id and must remove that \
             id from its host's `attachments`. Removing this call from ONE of the two \
             helpers reddens no behavioural probe in the workspace -- the bottom-of-library \
             sibling is `pub(crate)` and unreachable from an integration test -- which is \
             why this gate exists."
        );
        // The soulbond fix-up is the sibling this one is modelled on; if it ever leaves,
        // this gate's own justification ("two CR-400.7 hygiene steps sit together") is
        // stale and someone should notice here.
        assert!(
            body.contains("paired_with = None"),
            "the CR 702.95e soulbond fix-up is the precedent this one sits beside; if it \
             has moved out of `{site}`, re-read `detach_from_host_on_departure`'s doc"
        );
    }
}

/// The wrong-way-round half, as a SOURCE gate to match the behavioural one
/// (`pb_dx56_departure_hygiene::t3`): the helper must stay ONE-DIRECTIONAL.
///
/// CR 704.5m puts an illegally-attached Aura into its **owner's graveyard**; CR 704.5n
/// merely **unattaches** an Equipment or Fortification and leaves it on the battlefield.
/// Opposite dispositions for the same input, both already implemented as SBA arms — and
/// **CR 400.7f** exists specifically so a leaves-the-battlefield trigger can find an Aura
/// in its owner's graveyard *"as a result of being put there as a state-based action for
/// not being attached to a permanent. (See rule 704.5m.)"*, which is a rule whose
/// antecedent is that the Aura got there THROUGH 704.5m. So clearing `attached_to` here
/// would not merely be an SBA performed early; it would change which arm fires.
#[test]
fn r2_the_departure_fix_up_stays_one_directional() {
    let src = state_mod_src();
    let body = body_of(&src, "fn detach_from_host_on_departure(");
    assert!(
        body.contains("attachments.retain("),
        "non-vacuity: the helper must still be the one that prunes the host's list"
    );
    assert!(
        !body.contains("attached_to = None") && !body.contains("attached_to: None"),
        "CR 704.5m / CR 704.5n / CR 400.7f: this helper must NOT clear `attached_to` on \
         objects attached to a departing host. That is a state-based action with \
         type-dependent and OPPOSITE dispositions, already implemented in `rules/sba.rs`, \
         and CR 400.7f's finding rule depends on the Aura reaching the graveyard through \
         704.5m. See the wrong-way-round probe \
         `primitives::pb_dx56_departure_hygiene::t3`. Body: {body}"
    );
}

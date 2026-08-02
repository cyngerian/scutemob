/**
 * `plainClone` — the one sanctioned way to take a mutable, deep copy of a
 * server-sent DTO in this client.
 *
 * # Why this module exists (UI-4, `scutemob-185`; G1 of
 * `memory/playtest-triage-2026-08-02b.md`)
 *
 * Every answer-building picker starts from `template`: the engine's own default
 * answer, serialized verbatim by `view.rs` and handed down as a prop. The picker
 * copies it, writes one key, and posts it. The copy has to be deep, because the
 * key being written lives inside the externally-tagged enum's payload object,
 * and mutating the prop in place would write into the parent's reactive state.
 *
 * Three pickers reached for the platform's deep-copy primitive, and all three
 * were dead buttons for the entire life of the feature. `ActionBar.svelte`
 * declares `let activeOption = $state(null)` and assigns a plain store object
 * into it; Svelte 5 wraps that in a `Proxy` and deep-proxies on read, so by the
 * time `template` reaches a picker it is a proxy — and the structured-clone
 * algorithm rejects proxies outright, with:
 *
 *     DataCloneError: Failed to execute 'structuredClone' on 'Window':
 *     #<Object> could not be cloned.
 *
 * That throw escapes an ordinary Svelte 5 DOM handler. There was no `try` on the
 * emit path, so the DOM was untouched: the picker stayed open, no request was
 * issued, no error was shown. From the player's chair, Confirm did nothing.
 * Reproduced in headless Chromium against a live game before this fix was
 * written — the trace above is that run's, not a prediction.
 *
 * Five CR flows were dead behind it: library search (CR 701.23), scry
 * (CR 701.22a), surveil (CR 701.25a), sacrifice additional costs (CR 118.8) and
 * Squad (CR 702.157a).
 *
 * # Why `$state.snapshot`
 *
 * It is Svelte 5's own answer to exactly this problem: it unwraps reactive
 * proxies and returns a plain, mutable, deeply-copied value. It does not
 * re-serialize, so it cannot lose or coerce anything the wire put there, and it
 * is a no-op-shaped deep copy for values that were never reactive to begin with
 * — which matters because these components are also meant to be drivable from a
 * future test harness that passes plain objects (plan §8 R7).
 *
 * # The rule this module carries
 *
 * `tools/play-server/src/main.rs`'s
 * `test_frontend_never_structured_clones_reactive_state` fails the build if any
 * file under `frontend/src/` calls the platform primitive again, and asserts that
 * all three pickers call this function. The rule is a class, not three sites: the
 * same rejection hits `postMessage` and IndexedDB writes, so a Worker or a
 * persistence layer added later must route through here too.
 *
 * Its sibling, `test_frontend_picker_failures_reach_the_error_strip`, pins the
 * other half: the emit paths are guarded and their failures reach the player.
 *
 * @template T
 * @param {T} value a JSON-shaped value, reactive proxy or not
 * @returns {T} a plain deep copy safe to mutate and to serialize
 */
export function plainClone(value) {
  return $state.snapshot(value);
}

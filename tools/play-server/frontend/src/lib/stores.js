/**
 * Svelte stores for the play surface, plus the action helpers that own every
 * fetch.
 *
 * M11-local Session 6 (`memory/m11-session-plan.md` §4, item 3).
 *
 * # Request/response only — no WebSocket, no SSE
 *
 * Deliberate, and recorded as a milestone decision (`tools/play-server/README.md`
 * §"Decision record", `api.rs` module doc). The bots act **synchronously inside
 * the same request** that carries the human's action — `POST /api/game/action`
 * calls `submit` then `advance` and returns the state that results — so there is
 * never a moment where the server knows something the client has not been told in
 * a response it is already waiting for. A push channel is M10a's problem.
 *
 * # Components do not call `api.js`
 *
 * Every fetch goes through a helper here, so the `loading` / `error` bookkeeping
 * exists exactly once. `PlayApp` and `ActionBar` import the helpers, never the
 * adapter.
 */
import { get, writable } from 'svelte/store';
import * as api from './api.js';

/** The whole `SeatView` from the last successful response (null = no game). */
export const seatView = writable(null);

/** `SeatView.decision` — the decision the human must answer, or null. */
export const decision = writable(null);

/**
 * Rendered event lines, **accumulated**.
 *
 * The server returns only the lines since the client's last read
 * (`PlaySession::take_new_records` advances a journal cursor), so a naive
 * `set(view.events)` would discard the whole history on every request and leave
 * a feed that only ever shows the last command's output.
 */
export const events = writable([]);

/** True while a request is in flight. */
export const loading = writable(false);

/** `{ message, kind, status }` from the last failure, or null. */
export const error = writable(null);

/**
 * Event-history cap. A 200-turn game emits far more lines than anyone scrolls
 * back through, and the feed is a DOM node per line; bounding it keeps a long
 * session from growing without limit. The oldest lines are dropped, not the
 * newest — the interesting end is the bottom.
 *
 * **Raised 500 → 2000 by UI-3**, because that batch invalidated the number it
 * was chosen against. Before it, ~11 `GameEvent` variants rendered as prose and
 * everything else arrived as a bare kind string; now 60 do, so the lines *per
 * turn* went up several-fold and 500 no longer spans the recent history a
 * turn-grouped feed is meant to let you scroll back through. Adding the feature
 * that makes a cap bite and leaving the cap alone would have quietly truncated
 * the thing the feature exists to show.
 */
const MAX_EVENTS = 2000;

/**
 * Monotonic sequence stamped onto each event line as it is appended.
 *
 * `EventView` carries no id of its own (`{kind, text, player}` and nothing else,
 * deliberately — see its doc), and the feed is a **front-truncating** window, so
 * an array index is not a stable identity: once `MAX_EVENTS` engages, every
 * surviving line's index shifts by the number dropped and a keyed `#each` re-keys
 * the entire feed on every response. A counter that is assigned once and never
 * reused is the cheapest thing that stays stable. Reset with the feed.
 */
let nextEventSeq = 0;

function resetEvents() {
  nextEventSeq = 0;
  events.set([]);
  // An auto-pass status line describes a run against a table that no longer
  // exists — every caller of this is a redeal or a session that went away. It
  // also cancels: a loop still in flight when the game is replaced would keep
  // submitting passes into the new one.
  passUntilCancelled = true;
  passUntil.set(null);
}

/** Fold a `SeatView` into the stores. `seatView`/`decision` replace; `events` appends. */
function applySeatView(view) {
  seatView.set(view);
  decision.set(view?.decision ?? null);
  const incoming = view?.events ?? [];
  if (incoming.length > 0) {
    const stamped = incoming.map((ev) => ({ ...ev, seq: nextEventSeq++ }));
    events.update((prev) => [...prev, ...stamped].slice(-MAX_EVENTS));
  }
}

function toStoreError(err) {
  return {
    message: err?.message ?? String(err),
    kind: err?.kind ?? null,
    status: err?.status ?? null,
  };
}

/**
 * Run one request with the shared `loading`/`error` bookkeeping.
 *
 * **Concurrency guard**: a call made while another request is in flight is a
 * no-op. `ActionBar` also disables its buttons while `loading` is true; this is
 * the belt to that braces, because the keyboard shortcut and the click-through
 * path can both reach `act()` without touching a button, and two submissions
 * racing on one `seq` means the second is answered with a 409 that the user did
 * nothing to deserve.
 *
 * Returns true on success, false on failure or when the guard rejected the call.
 */
async function run(call, { clearFeed = false } = {}) {
  if (get(loading)) return false;
  loading.set(true);
  try {
    const view = await call();
    if (clearFeed) resetEvents();
    applySeatView(view);
    error.set(null);
    return true;
  } catch (err) {
    error.set(toStoreError(err));
    return false;
  } finally {
    loading.set(false);
  }
}

/**
 * Start (or restart) a game. Clears the feed: the previous game's lines describe
 * a table that no longer exists.
 */
export function startGame(opts = {}) {
  return run(() => api.newGame(opts), { clearFeed: true });
}

/**
 * Re-read this seat's view.
 *
 * A 404 `no_session` is **not** an error to shout about — it is the ordinary
 * state before the first `POST /api/game`, and `onMount` hits it every time the
 * page is opened cold. It clears the stores instead, which is what `PlayApp`
 * renders its "start a game" empty state from (`seatView === null && !error`).
 */
export async function refresh() {
  if (get(loading)) return false;
  loading.set(true);
  try {
    applySeatView(await api.getGame());
    error.set(null);
    return true;
  } catch (err) {
    if (err?.kind === 'no_session' || err?.status === 404) {
      seatView.set(null);
      decision.set(null);
      resetEvents();
      error.set(null);
      return false;
    }
    error.set(toStoreError(err));
    return false;
  } finally {
    loading.set(false);
  }
}

/**
 * Answer the pending decision with `actionIndex` (an index into
 * `decision.actions`, which is the entire submission protocol — a `LegalAction`
 * never crosses the wire, see `view.rs`'s module doc).
 *
 * `seq` is read from the store rather than passed in, so a caller cannot pair an
 * index from one decision with the `seq` of another.
 */
export async function act(actionIndex, params = {}) {
  const pending = get(decision);
  if (!pending) {
    error.set({
      message: 'there is no decision awaiting an answer',
      kind: 'no_pending_decision',
      status: null,
    });
    return false;
  }
  return run(() => api.submitAction(pending.seq, actionIndex, params));
}

/**
 * CR 103.5 — take a pregame mulligan.
 *
 * Goes through the dedicated `POST /api/game/mulligan` route, not through
 * `act()`; see `PlayApp.svelte`'s pregame comment for why that is the only path
 * that works. The redeal rebuilds the whole table, so the feed is cleared for the
 * same reason `startGame` clears it.
 */
export function takeMulligan() {
  return run(() => api.mulligan(true), { clearFeed: true });
}

/**
 * Keep the hand as dealt. Server-side this is a no-op that just re-renders the
 * seat view (`post_mulligan` skips the rebuild when `take` is false), so the feed
 * is preserved.
 */
export function keepHand() {
  return run(() => api.mulligan(false));
}

/** Dismiss the error strip without issuing a request. */
export function dismissError() {
  error.set(null);
}

/**
 * Put a **client-side** failure on the same error strip the server errors use.
 *
 * UI-4 (`scutemob-185`; G1 of `memory/playtest-triage-2026-08-02b.md`). Every
 * other writer of this store is a failed HTTP call, so `kind` came from
 * `ApiError.kind` and `status` from the response. A client-side throw has
 * neither, and `'client_error'` is a kind `api.rs` cannot produce — which is the
 * point: the strip must be able to say "this went wrong in your browser, not on
 * the server", because those are two different bugs to report.
 *
 * Introduced because a picker threw a `DataCloneError` out of a click handler
 * for the entire life of the feature and *nothing on screen changed*. A silent
 * throw is what turned a three-line bug into a conceded game.
 */
export function reportClientError(message) {
  error.set({
    message: String(message ?? 'the client hit an unexpected problem'),
    kind: 'client_error',
    status: null,
  });
}

/**
 * The net under every handler, not just the ones with a `try`.
 *
 * A DOM event handler that throws does not fail loudly in a browser: the
 * exception unwinds to the platform, which logs it to a console nobody has open
 * and leaves the DOM exactly as it was. Svelte 5's `<svelte:boundary>` does not
 * help — it catches render and effect errors, not handler ones. The only place
 * that sees all of them is `window`.
 *
 * So the per-picker `try/catch` blocks buy a *specific* message, and this buys
 * the guarantee: after UI-4 there is no way for a click in this client to fail
 * and leave the screen unchanged. Called once from `main.js`; idempotent so a
 * hot reload cannot stack listeners.
 */
let globalHandlersInstalled = false;
export function installGlobalErrorReporting(target = globalThis) {
  if (globalHandlersInstalled || !target?.addEventListener) return;
  globalHandlersInstalled = true;
  target.addEventListener('error', (event) => {
    const err = event?.error;
    reportClientError(
      `unhandled ${err?.name ?? 'error'} in the client: ${err?.message ?? event?.message ?? 'no detail'}`,
    );
  });
  target.addEventListener('unhandledrejection', (event) => {
    const err = event?.reason;
    reportClientError(
      `unhandled ${err?.name ?? 'rejection'} in the client: ${err?.message ?? String(err ?? 'no detail')}`,
    );
  });
}

// ── Pass-until (UI-3, `scutemob-180`, AC 6009) ───────────────────────────────

/**
 * Auto-pass status: `{ mode, passes, stopReason }` while a run is in flight or
 * has just finished, `null` when idle.
 *
 * `stopReason` is null while running and a sentence when it has stopped, so the
 * UI can always say *why* it gave you priority back. "It stopped" with no reason
 * is the failure mode of every auto-pass button in every client, and the answer
 * ("something was cast", "it is your turn", "you were asked to discard") is the
 * only interesting part.
 */
export const passUntil = writable(null);

/**
 * Cancellation flag for the running loop.
 *
 * A plain module-level boolean rather than a store read: the loop must see the
 * cancel the moment it is set, and `get(store)` in the loop body would do the
 * same job with more ceremony. Reset at the start of every run.
 */
let passUntilCancelled = false;

/**
 * Hard bound on one run. A pass costs a round trip, so this is a stall guard,
 * not a policy: a table that legitimately needs more than this many consecutive
 * human passes has something wrong with it, and spinning forever against a
 * server that keeps handing priority back is worse than stopping and saying so.
 *
 * Sized against the server's own limits (`session.rs::config_for` allows 500
 * consecutive passes and 200 turns), so this stops first and stops visibly.
 */
const MAX_AUTO_PASSES = 400;

/**
 * The stop predicates.
 *
 * Each takes `(view, ctx)` — the `SeatView` just received and the context
 * captured when the run started — and returns a **reason string** to stop, or
 * `null` to keep passing.
 *
 * # Shape, and why it is this shape
 *
 * The playtest note asks for two buttons now and names the generalisation it
 * wants later: "this could be fine grained in the future … select player turn
 * and phase and the priority will pass until that phase (ex: Bot-3 end)". So a
 * mode is an object, not a string constant, and the dispatch is on `mode.kind`
 * with the rest of the object free for parameters. Adding that third mode is
 * one entry here plus one control in `ActionBar`; nothing else moves.
 *
 * Everything is read from the seat-redacted `SeatView` the server already sends.
 * There is no server change for this feature and no new route — `POST
 * /api/game/action` with the existing `PassPriority` index is the whole
 * mechanism, which is exactly what a human clicking "pass" repeatedly does.
 */
const PASS_UNTIL_PREDICATES = {
  /**
   * "Pass until my turn starts."
   *
   * Stops the moment the active player (CR 102.1 / 500.1) is this seat. Note
   * this is the *active player*, not priority: you get control back at the top
   * of your own turn rather than at the first priority you receive during it.
   */
  'my-turn': (view, ctx) => {
    const active = view?.state?.turn?.active_player ?? null;
    if (active !== null && active === ctx.humanName) return 'your turn began';
    return null;
  },

  /**
   * "Pass until something happens, or the phase ends."
   *
   * Two stop conditions, both meaning "the board is not what it was when you
   * stopped paying attention":
   *
   *   - the step or phase changed (CR 500.1 — the turn moved on), or
   *   - the stack grew (CR 405.1 — somebody cast or activated something).
   *
   * The stack check compares against the depth seen on the **previous
   * iteration**, not against the depth when the run started, and the difference
   * is real: with a fixed baseline, a spell that resolved off the stack and a
   * different one that went on in its place is a net-zero change and would be
   * passed straight through. Any growth from one response to the next stops the
   * run.
   *
   * `ctx.step`/`ctx.phase` stay fixed at the starting values on purpose — the
   * question there is "has the turn moved on from where I stopped paying
   * attention", which is a comparison against the start, not against the last
   * poll.
   */
  'phase-end': (view, ctx) => {
    const step = view?.state?.turn?.step ?? null;
    const phase = view?.state?.turn?.phase ?? null;
    if (step !== ctx.step || phase !== ctx.phase) {
      return `the ${ctx.phase} phase moved on (${ctx.step} → ${step})`;
    }
    const depth = view?.state?.zones?.stack?.length ?? 0;
    const grew = depth > ctx.stackDepth;
    ctx.stackDepth = depth;
    return grew ? 'something was put on the stack' : null;
  },
};

/** Cancel a running auto-pass. Safe to call when nothing is running. */
export function cancelPassUntil() {
  passUntilCancelled = true;
}

/**
 * Pass priority repeatedly until `modeKind`'s predicate says to stop.
 *
 * **Entirely client-side.** Every iteration is one ordinary `POST
 * /api/game/action` naming the `PassPriority` option the server already offered,
 * so the server cannot tell this apart from a human clicking pass quickly, and
 * no recorded seed or replay is affected.
 *
 * Unconditional stop conditions, checked before the predicate on every
 * iteration — these are the ones that make the loop safe rather than merely
 * convenient:
 *
 *   - the user cancelled;
 *   - the game ended (`game_over`);
 *   - there is no decision to answer;
 *   - the decision is **not** a `Priority` one. This is the important one: a
 *     cleanup discard, a trigger's targets, a blocker declaration and a search
 *     are all real choices (UI-1's whole subject), and an auto-passer that
 *     answered them with a default would be making the human's decisions for
 *     them — the exact defect UI-1 existed to remove. The loop hands control
 *     back instead;
 *   - the current decision offers no `PassPriority` action at all;
 *   - a request failed (`act` returns false, having set `error`);
 *   - `MAX_AUTO_PASSES` iterations.
 */
export async function startPassUntil(modeKind) {
  const predicate = PASS_UNTIL_PREDICATES[modeKind];
  if (!predicate) {
    error.set({
      message: `unknown pass-until mode "${modeKind}"`,
      kind: 'client_bug',
      status: null,
    });
    return false;
  }
  // Refuse a second concurrent run. `loading` alone is not enough: it is false
  // between iterations, so a second call landing in that window would start a
  // rival loop, both would drive the same session, and `passUntilCancelled`
  // would stop whichever checked it first while the other kept passing. The
  // buttons are already hidden mid-run (`ActionBar`'s `autoPassing`); this is
  // the guard behind that, for any caller that is not a button.
  const running = get(passUntil);
  if (running && running.stopReason === null) return false;
  if (get(loading)) return false;

  const start = get(seatView);
  if (!start) return false;

  const ctx = {
    humanName: start.summary?.human_name ?? null,
    phase: start.state?.turn?.phase ?? null,
    step: start.state?.turn?.step ?? null,
    stackDepth: start.state?.zones?.stack?.length ?? 0,
  };

  passUntilCancelled = false;
  let passes = 0;
  passUntil.set({ mode: modeKind, passes, stopReason: null });

  const finish = (stopReason) => {
    passUntil.set({ mode: modeKind, passes, stopReason });
    return true;
  };

  for (;;) {
    if (passUntilCancelled) return finish('cancelled');
    if (passes >= MAX_AUTO_PASSES) {
      return finish(`stopped after ${MAX_AUTO_PASSES} passes — this looks stuck`);
    }

    const view = get(seatView);
    if (!view) return finish('the game went away');
    if (view.game_over) return finish('the game is over');

    const pending = get(decision);
    if (!pending) return finish('there is nothing to answer');
    if (pending.kind !== 'Priority') {
      return finish(`you have a ${pending.kind} decision to make`);
    }

    // Only after the loop has passed at least once do we let the predicate stop
    // it — otherwise "pass until my turn" would refuse to start on your own
    // turn, which is the one time you most want to pass through to the end of
    // it. The unconditional guards above still apply on iteration 0.
    if (passes > 0) {
      const reason = predicate(view, ctx);
      if (reason) return finish(reason);
    }

    const pass = (pending.actions ?? []).find((a) => a.kind === 'PassPriority');
    if (!pass) return finish('passing is not offered right now');

    const ok = await act(pass.index, {});
    if (!ok) return finish('a request failed — see the error above');
    passes += 1;
    passUntil.set({ mode: modeKind, passes, stopReason: null });
  }
}

/** Clear the finished-run status line. */
export function dismissPassUntil() {
  passUntil.set(null);
}

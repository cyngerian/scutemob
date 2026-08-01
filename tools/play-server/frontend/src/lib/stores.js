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
 */
const MAX_EVENTS = 500;

/** Fold a `SeatView` into the stores. `seatView`/`decision` replace; `events` appends. */
function applySeatView(view) {
  seatView.set(view);
  decision.set(view?.decision ?? null);
  const incoming = view?.events ?? [];
  if (incoming.length > 0) {
    events.update((prev) => [...prev, ...incoming].slice(-MAX_EVENTS));
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
async function run(call, { resetEvents = false } = {}) {
  if (get(loading)) return false;
  loading.set(true);
  try {
    const view = await call();
    if (resetEvents) events.set([]);
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
  return run(() => api.newGame(opts), { resetEvents: true });
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
      events.set([]);
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
  return run(() => api.mulligan(true), { resetEvents: true });
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

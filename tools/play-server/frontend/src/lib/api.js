/**
 * API fetch wrapper for the play server (`tools/play-server/src/api.rs`).
 *
 * M11-local Session 6 (`memory/m11-session-plan.md` §4, item 3). Same shape as
 * the replay viewer's `tools/replay-viewer/frontend/src/lib/api.js`: an empty
 * relative BASE so the dev Vite proxy (`vite.config.js` -> 127.0.0.1:3040) and
 * the production `ServeDir` mount both work with no configuration.
 *
 * Every function returns the parsed `SeatView` or throws.
 *
 * # Errors carry `status` and `kind`, not just prose
 *
 * `api.rs` answers every handler failure in one envelope,
 * `{"error": "...", "kind": "..."}`, precisely so a client can branch without
 * parsing prose (`ApiError`). The thrown `Error` therefore carries `.status` and
 * `.kind` alongside `.message`, and `ActionBar`/`stores.js` branch on `.kind`
 * (`rejected` = the engine refused the play, `stale_decision` = resync,
 * `no_session` = no game yet).
 *
 * **Two failures carry no envelope at all**: an unmatched path (404) and a wrong
 * method (405) come from axum's router, not from a handler, so the body is empty
 * and there is no `Content-Type`. Parsing must not throw on those — hence the
 * text-then-try-parse shape below rather than a bare `response.json()`.
 */

const BASE = '';

/** JSON POST options; the play server's DTOs all require the JSON content type. */
function jsonPost(body) {
  return {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  };
}

/**
 * Build the `Error` for a non-2xx response.
 *
 * Falls back through three shapes: the JSON envelope, a non-JSON body (axum's
 * own `text/plain` rejections), and an empty body (the router's 404/405).
 */
function apiError(path, status, raw) {
  let message = null;
  let kind = null;
  try {
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed.error === 'string') {
      message = parsed.error;
      kind = typeof parsed.kind === 'string' ? parsed.kind : null;
    }
  } catch {
    // Not the envelope — fall through to the raw-text / status-only fallbacks.
  }
  if (message === null) {
    const trimmed = (raw ?? '').trim();
    message = trimmed ? `HTTP ${status}: ${trimmed}` : `HTTP ${status}`;
  }
  const err = new Error(message);
  err.status = status;
  err.kind = kind;
  err.path = path;
  return err;
}

async function apiFetch(path, options = {}) {
  const response = await fetch(`${BASE}${path}`, options);
  // Read as text first: a router-level failure has an empty body, and
  // `response.json()` would throw a `SyntaxError` that hides the real status.
  const raw = await response.text().catch(() => '');
  if (!response.ok) {
    throw apiError(path, response.status, raw);
  }
  return raw ? JSON.parse(raw) : null;
}

/**
 * POST /api/game — start (or restart) a game. Returns `SeatView`.
 *
 * Fields are omitted rather than sent as `null` when unset: `NewGameRequest` is
 * `deny_unknown_fields` + `Option<T>` per field, so an omitted key takes the
 * server's CLI default.
 */
export function newGame({ players, bot, seed } = {}) {
  const body = {};
  if (players !== undefined && players !== null) body.players = players;
  if (bot !== undefined && bot !== null) body.bot = bot;
  if (seed !== undefined && seed !== null) body.seed = seed;
  return apiFetch('/api/game', jsonPost(body));
}

/** GET /api/game — this seat's view. 404 `no_session` when no game is running. */
export function getGame() {
  return apiFetch('/api/game');
}

/**
 * POST /api/game/action — answer the pending decision.
 *
 * `seq` is the **wire** seq from `decision.seq` (`PlaySession::wire_seq`); a
 * mismatch is a 409 `stale_decision`. `params` mirrors `ActionParamsDto`, whose
 * every field defaults, so `{}` is valid for an action that announces nothing.
 * Session 7 fills it in for targets / attackers / blockers / X / modes.
 */
export function submitAction(seq, actionIndex, params = {}) {
  return apiFetch(
    '/api/game/action',
    jsonPost({ seq, action_index: actionIndex, params }),
  );
}

/**
 * POST /api/game/mulligan — CR 103.5 pregame redeal. Pregame only (409
 * `not_pregame` afterwards).
 *
 * `cards_to_bottom` is deliberately never sent: `api.rs::post_mulligan` refuses a
 * non-empty list with 400, because the whole-table rebuild leaves the engine's
 * `PlayerState::mulligan_count` at 0 and CR 103.5's bottoming half is not
 * expressible on this path.
 */
export function mulligan(take) {
  return apiFetch('/api/game/mulligan', jsonPost({ take }));
}

/**
 * GET /api/game/report — the bug-report / repro artefact (M11-local Session 8,
 * item 5; `docs/mtg-engine-runtime-integrity.md` Layer 3).
 *
 * Returns the parsed `BugReportView`, **not** a `SeatView` — the only function in
 * this module that does. A pure read: it neither advances the game nor consumes
 * the event lines `getGame()` has not delivered yet, so it is safe to call while a
 * decision is outstanding.
 *
 * Note the payload is **not** seat-redacted: it carries every seat's raw
 * `GameEvent`s deliberately, because a redacted repro cannot be replayed. That is
 * safe for M11-local (one human, three bots, one process, no networking) and is
 * exactly what has to be revisited at M10a — see `view.rs`'s `BugReportView` doc.
 */
export function getReport() {
  return apiFetch('/api/game/report');
}

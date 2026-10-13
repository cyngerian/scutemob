/**
 * order — the client-side order of your own cards (`docs/play-surface-v3-brief.md` §2 item 3:
 * "Drag to reorder cards in your hand and on your battlefield … Client-side order,
 * remembered per game").
 *
 * Two halves, both pure JavaScript (no Svelte, no DOM) so plain `node` can exercise them;
 * the pointer mechanics are `reorder.js`, the rows are `HandRow` and `YourBoard`.
 *
 * ## Which game is this? (`trackGame`, `newGameKey`)
 *
 * The seat view's `summary` carries no game id and — deliberately, Invariant 7 — no seed.
 * So the key is kept here: `localStorage['v3-game']` holds
 * `{ key, players, human_name, bot, mulligan_count, turn, command_count }`, and
 * `trackGame(summary)` answers that `key` for as long as the summary is consistent with
 * being the same table: the same `players` / `human_name` / `bot` / `mulligan_count`, and
 * `turn` / `command_count` not lower than last seen (the stored counters follow the game
 * forward). Anything else mints a new key. `newGameKey()` drops the record, which
 * `TableV3` calls before it deals or mulligans.
 *
 * **Known limitation.** A different game started somewhere else (another tab, a server
 * restart, a probe) that has already run PAST the stored counters is indistinguishable
 * from this one, and keeps the key. A stale order is then applied to whatever object ids
 * happen to match. That is cosmetic — the order of your own cards on your own screen,
 * never a rules matter, and one drag overwrites it — but it is a guess. The real fix is
 * a `game_id` on `GameSummary`, a server change that belongs to C3 (`scutemob-268`).
 *
 * ## The order (`applyOrder`, `moveItem`, `moveItems`, `loadOrder`, `saveOrder`)
 *
 * Per game key and per list (`LISTS`: your hand and the three battlefield rows) a saved
 * array of object ids, in `localStorage['v3-order:<gameKey>']`.
 *
 *   - **New objects append**: `applyOrder` puts the saved ids first, in saved order, then
 *     everything else in the order the seat view sent it.
 *   - **A card that leaves drops out**: an id that is no longer in the list is ignored,
 *     and gone from the array the next time the row is saved (a save is always the ids
 *     now on screen). CR 400.7: a card that leaves and comes back is a NEW object with a
 *     new id, so it does not get its old place back — it appends like any new object.
 *   - Lands are saved as a flat list of ids, not of stacks: a stack's key changes when a
 *     land taps, its members' ids do not. `YourBoard` orders the list first and stacks
 *     after; moving a stack is `moveItems` with every member.
 *
 * Storage is a convenience: every access is wrapped, and without it the table still
 * works (the game key lives in this module for the life of the page; orders are simply
 * not remembered across a reload).
 */

const GAME_KEY = 'v3-game';
const ORDER_PREFIX = 'v3-order:';
/** How many games' orders are kept; older ones are pruned on save. */
const KEEP_GAMES = 8;

/** The reorderable lists: your hand and the three rows of your battlefield. */
export const LISTS = ['hand', 'creatures', 'others', 'lands'];

/** The page-lifetime copy of the game record, the only one when storage is unavailable. */
let memoryGame = null;

function storage() {
  try {
    return globalThis.localStorage ?? null;
  } catch {
    return null; // a sandboxed frame throws on the getter itself
  }
}

function readJson(key) {
  try {
    const raw = storage()?.getItem(key);
    return raw ? JSON.parse(raw) : null;
  } catch {
    return null;
  }
}

function writeJson(key, value) {
  try {
    const store = storage();
    if (!store) return false;
    store.setItem(key, JSON.stringify(value));
    return true;
  } catch {
    return false; // full, or denied: the caller carries on without it
  }
}

function removeKey(key) {
  try {
    storage()?.removeItem(key);
  } catch {
    // nothing to remove from
  }
}

// ── Which game ───────────────────────────────────────────────────────────────

const count = (v) => (Number.isFinite(Number(v)) ? Number(v) : 0);

/** The summary fields that never change during one game. */
function identity(summary) {
  return JSON.stringify([
    summary?.players ?? null,
    summary?.human_name ?? null,
    summary?.bot ?? null,
    summary?.mulligan_count ?? null,
  ]);
}

function sameTable(record, summary) {
  if (!record || typeof record.key !== 'string' || record.key === '') return false;
  if (identity(record) !== identity(summary)) return false;
  return count(summary.turn) >= count(record.turn) && count(summary.command_count) >= count(record.command_count);
}

function mintKey() {
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
}

/**
 * The key the order of this game's cards is remembered under. Stable while the summary
 * keeps looking like the same table (see the header for when that is a guess).
 */
export function trackGame(summary) {
  const stored = readJson(GAME_KEY) ?? memoryGame;
  const same = sameTable(stored, summary);
  const record = {
    key: same ? stored.key : mintKey(),
    players: summary?.players ?? null,
    human_name: summary?.human_name ?? null,
    bot: summary?.bot ?? null,
    mulligan_count: summary?.mulligan_count ?? null,
    turn: count(summary?.turn),
    command_count: count(summary?.command_count),
  };
  // written only when something moved: this runs on every seat view
  if (!same || record.turn !== count(stored.turn) || record.command_count !== count(stored.command_count)) {
    writeJson(GAME_KEY, record);
  }
  memoryGame = record;
  return record.key;
}

/** This client is about to deal (or mulligan into) a new table: the next `trackGame` mints. */
export function newGameKey() {
  memoryGame = null;
  removeKey(GAME_KEY);
}

// ── The order ────────────────────────────────────────────────────────────────

const objectId = (c) => Number(c.object_id);

/**
 * `items` in the saved order: the items whose id is in `savedIds`, in that order, then
 * every other item in its incoming order. Saved ids with no item are skipped.
 */
export function applyOrder(items, savedIds, idOf = objectId) {
  const list = items ?? [];
  if (!Array.isArray(savedIds) || savedIds.length === 0) return list;
  const byId = new Map();
  for (const item of list) {
    const id = idOf(item);
    // an item with no usable id (or a repeated one) is never "saved": it keeps its incoming place at the end
    if (Number.isFinite(id) && !byId.has(id)) byId.set(id, item);
  }
  const placed = new Set();
  const out = [];
  for (const raw of savedIds) {
    const id = Number(raw);
    const item = byId.get(id);
    if (item === undefined || placed.has(item)) continue;
    placed.add(item);
    out.push(item);
  }
  for (const item of list) if (!placed.has(item)) out.push(item);
  return out;
}

/**
 * `ids` with `movedIds` (kept in their current relative order) taken out and put back
 * together just before `beforeId` — at the end when `beforeId` is null or not in the
 * list. A `beforeId` that is itself being moved leaves the group where its first member
 * stood. Ids in `movedIds` that are not in `ids` are ignored.
 */
export function moveItems(ids, movedIds, beforeId = null) {
  const list = (ids ?? []).map(Number);
  const moving = new Set((movedIds ?? []).map(Number));
  const moved = list.filter((id) => moving.has(id));
  if (moved.length === 0) return list;
  let before = beforeId === null || beforeId === undefined ? null : Number(beforeId);
  if (before !== null && moving.has(before)) {
    // dropped on itself: the anchor is the first id after the group's own first member that is staying
    before = list.slice(list.indexOf(moved[0])).find((id) => !moving.has(id)) ?? null;
  }
  const rest = list.filter((id) => !moving.has(id));
  const at = before === null ? -1 : rest.indexOf(before);
  if (at < 0) return [...rest, ...moved];
  return [...rest.slice(0, at), ...moved, ...rest.slice(at)];
}

/** One id moved to just before `beforeId` (null = the end). */
export function moveItem(ids, movedId, beforeId = null) {
  return moveItems(ids, [movedId], beforeId);
}

const cleanIds = (v) => (Array.isArray(v) ? v.map(Number).filter((id) => Number.isFinite(id)) : []);

/** The saved lists of `gameKey` — always every list of `LISTS`, empty when nothing is saved. */
export function loadOrder(gameKey) {
  const saved = gameKey ? readJson(ORDER_PREFIX + gameKey) : null;
  const out = {};
  for (const name of LISTS) out[name] = cleanIds(saved?.[name]);
  return out;
}

/** Drop the orders of all but the `KEEP_GAMES` most recently saved games. */
function pruneOrders(keepKey) {
  try {
    const store = storage();
    if (!store) return;
    const found = [];
    for (let i = 0; i < store.length; i++) {
      const key = store.key(i);
      if (!key || !key.startsWith(ORDER_PREFIX) || key === keepKey) continue;
      found.push({ key, at: count(readJson(key)?.at) });
    }
    found.sort((a, b) => b.at - a.at);
    for (const { key } of found.slice(KEEP_GAMES - 1)) store.removeItem(key);
  } catch {
    // pruning is housekeeping; a failure costs a few stale keys
  }
}

/**
 * Save `lists` (any subset of `LISTS` → id arrays) for `gameKey`, leaving the lists not
 * named as they were: the hand row and the board each save only their own. Returns false
 * when nothing was stored (no key, no storage).
 */
export function saveOrder(gameKey, lists) {
  if (!gameKey) return false;
  const next = { ...loadOrder(gameKey), at: Date.now() };
  for (const name of LISTS) if (lists && name in lists) next[name] = cleanIds(lists[name]);
  const ok = writeJson(ORDER_PREFIX + gameKey, next);
  if (ok) pruneOrders(ORDER_PREFIX + gameKey);
  return ok;
}

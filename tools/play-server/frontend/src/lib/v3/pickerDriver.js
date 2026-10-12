/**
 * pickerDriver — drives the mounted `TargetPicker` through its own DOM
 * (`docs/play-surface-v3-brief.md` §2 items 1 and 2).
 *
 * An arrow or a context-menu item must submit EXACTLY what the picker would.
 * The only way to guarantee that without touching `PromptPanel` or
 * `TargetPicker` is to not submit anything here at all: this module clicks the
 * picker's own `.candidate` buttons and its own `.confirm` button, so the
 * announcement that reaches the server is the one `TargetPicker.confirm` builds
 * (`slot.candidates[c].value`, grouped then flattened), through the one chain
 * `PromptPanel` owns. Nothing in `lib/v3` ever writes a `targets` parameter.
 *
 * What it has to work out is WHICH button stands for a given candidate.
 * `PromptPanel` does not expose its chain state, and a chain can be started
 * from inside it (the auto-open of a lone blocking answer, a loose button), so
 * the answer is read back from what is on screen:
 *
 *   1. `.stage-line .stage-name` is the active option's `label`;
 *   2. the picker's `.slot`s, each with its ordered button labels, are its
 *      signature;
 *   3. an option can feed the picker from several slot lists (`target_slots`,
 *      `fused_target_slots`, a trigger's `decision.answer.slots`, the chosen
 *      modes' `target_slots` concatenated) — the one whose signature equals the
 *      DOM's, slot for slot and label for label, is the one being shown.
 *
 * The order of the buttons inside a slot is `TargetPicker.segmentsFor`'s and is
 * replicated in `displayOrder`. Every click re-checks that the button's text is
 * the candidate's label. When nothing matches, or two matches disagree about
 * which object a button stands for, `resolvePicker` returns null and arrows are
 * simply not offered for that picker — the list still works. Never a guess.
 *
 * Pure DOM + data: no Svelte state, no fetch.
 */

/** More modes than this and the subset search is not attempted (2^n lists). */
const MAX_MODES = 6;

/** `TargetPicker`'s key for the segment of candidates with no `owner`. */
const NO_OWNER = ' no-owner';

const text = (node) => (node?.textContent ?? '').replace(/\s+/g, ' ').trim();
const norm = (label) => String(label ?? '').replace(/\s+/g, ' ').trim();

/** The mounted target picker inside the decision host, or null. */
export function findPicker(host) {
  return host?.querySelector?.('.picker-area .target-picker') ?? null;
}

/** Is a picker chain (any stage) open in the decision host? */
export function chainOpen(host) {
  return !!host?.querySelector?.('.picker-area');
}

/** Is a target picker mounted in the decision host right now? */
export function isLive(host) {
  return findPicker(host) !== null;
}

/**
 * The candidate indices of one slot in the order `TargetPicker` draws their
 * buttons: grouped by `owner`; the human's segment first, the other owners in
 * first-appearance order, the owner-less segment last; original index order
 * inside a segment. (`Array.prototype.sort` is stable, which is what
 * `segmentsFor` relies on too.)
 */
export function displayOrder(slot, humanName = null) {
  const byOwner = new Map();
  (slot?.candidates ?? []).forEach((candidate, index) => {
    const key = candidate?.owner ?? NO_OWNER;
    if (!byOwner.has(key)) byOwner.set(key, []);
    byOwner.get(key).push(index);
  });
  const keys = [...byOwner.keys()];
  keys.sort((a, b) => {
    if (a === b) return 0;
    if (a === NO_OWNER) return 1;
    if (b === NO_OWNER) return -1;
    if (humanName !== null) {
      if (a === humanName) return -1;
      if (b === humanName) return 1;
    }
    return 0;
  });
  return keys.flatMap((key) => byOwner.get(key));
}

/** Every k-subset of `items`, k in `min..max`, each in the items' own order. */
function subsets(items, min, max) {
  const out = [];
  const walk = (start, chosen) => {
    if (chosen.length >= min && chosen.length <= max) out.push([...chosen]);
    if (chosen.length >= max) return;
    for (let i = start; i < items.length; i++) {
      chosen.push(items[i]);
      walk(i + 1, chosen);
      chosen.pop();
    }
  };
  walk(0, []);
  return out;
}

/**
 * Every slot list the picker could be showing for `option`. Mirrors
 * `PromptPanel.resolvedTargetSlots` (fused, per-mode, plain) plus the `Slots`
 * decision stage, without knowing which the chain is in.
 */
export function slotListsFor(option) {
  const lists = [];
  const push = (slots) => {
    if (Array.isArray(slots) && slots.length > 0) lists.push(slots);
  };
  push(option?.target_slots);
  push(option?.fused_target_slots);
  if (option?.decision?.answer?.shape === 'Slots') push(option.decision.answer.slots);
  const modes = option?.modes ?? [];
  if (modes.some((m) => (m.target_slots?.length ?? 0) > 0) && modes.length <= MAX_MODES) {
    const ordered = [...modes].sort((a, b) => a.index - b.index);
    const min = Math.max(1, option.mode_min ?? 1);
    const max = Math.max(min, option.mode_max ?? ordered.length);
    for (const chosen of subsets(ordered, min, max)) push(chosen.flatMap((m) => m.target_slots ?? []));
  }
  return lists;
}

/**
 * Every candidate an option could ever announce, across all of its slot lists:
 * `{ objects: Set<number>, players: Set<string> }`. A player candidate is keyed
 * by `owner`, which for a player target is that player's display name
 * (`TargetOptionView.owner`), i.e. the `data-seat` of their containers.
 */
export function candidatesOf(option) {
  const objects = new Set();
  const players = new Set();
  const read = (slots) => {
    for (const slot of slots ?? []) {
      for (const c of slot?.candidates ?? []) {
        if (c?.kind === 'object') objects.add(Number(c.id));
        else if (c?.kind === 'player' && c.owner) players.add(c.owner);
      }
    }
  };
  read(option?.target_slots);
  read(option?.fused_target_slots);
  for (const mode of option?.modes ?? []) read(mode.target_slots);
  if (option?.decision?.answer?.shape === 'Slots') read(option.decision.answer.slots);
  return { objects, players };
}

/** Does `candidate` stand for `pick` (`{ kind: 'object', id }` | `{ kind: 'player', name }`)? */
export function matchesPick(candidate, pick) {
  if (!candidate || !pick) return false;
  if (pick.kind === 'object') return candidate.kind === 'object' && Number(candidate.id) === Number(pick.id);
  if (pick.kind === 'player') return candidate.kind === 'player' && candidate.owner === pick.name;
  return false;
}

/** What the picker's DOM shows: per `.slot`, its buttons and their labels in order. */
function readDom(picker) {
  return [...picker.querySelectorAll('.slots > .slot')].map((slot) => {
    const buttons = [...slot.querySelectorAll('button.candidate')];
    return { buttons, labels: buttons.map(text) };
  });
}

/** Per slot, the candidate index behind each button position — or null if the labels differ. */
function mapSlots(slots, dom, humanName) {
  if (slots.length !== dom.length) return null;
  const orders = [];
  for (let s = 0; s < slots.length; s++) {
    const order = displayOrder(slots[s], humanName);
    if (order.length !== dom[s].labels.length) return null;
    for (let k = 0; k < order.length; k++) {
      if (norm(slots[s].candidates[order[k]].label) !== dom[s].labels[k]) return null;
    }
    orders.push(order);
  }
  return orders;
}

/** What each button stands for, as one string — two matches must agree on it. */
function identity(slots, orders) {
  return slots
    .map((slot, s) => orders[s].map((c) => `${slot.candidates[c].kind}:${slot.candidates[c].id}:${JSON.stringify(slot.candidates[c].value)}`).join(','))
    .join('|');
}

/**
 * Work out which option and which slot list the mounted target picker is
 * showing. Returns
 *   `{ picker, option, slots, buttons }` — `buttons[slot][candidateIndex]` is
 *     the `.candidate` element for `slots[slot].candidates[candidateIndex]`;
 *   `null` when no target picker is mounted, or when the DOM cannot be matched
 *     to exactly one reading of the decision payload.
 */
export function resolvePicker(host, actions, humanName = null) {
  const picker = findPicker(host);
  if (!picker) return null;
  const label = text(host.querySelector('.picker-area .stage-line .stage-name'));
  const dom = readDom(picker);
  if (dom.length === 0) return null;
  let found = null;
  let foundIdentity = null;
  for (const option of actions ?? []) {
    if (norm(option.label) !== label) continue;
    for (const slots of slotListsFor(option)) {
      const orders = mapSlots(slots, dom, humanName);
      if (!orders) continue;
      const id = identity(slots, orders);
      if (found === null) {
        found = { options: [option], slots, orders };
        foundIdentity = id;
      } else if (id !== foundIdentity) {
        // Two readings put different objects behind the same buttons.
        return null;
      } else if (!found.options.includes(option)) {
        // Same buttons, same objects, but two options carry this label (two
        // copies of one spell). Which copy the chain belongs to is unknowable
        // from here and does not matter to a click: the buttons are exact for
        // either. All of them are reported.
        found.options.push(option);
      }
    }
  }
  if (!found) return null;
  const buttons = found.slots.map((slot, s) => {
    const row = new Array(slot.candidates.length).fill(null);
    found.orders[s].forEach((candidateIndex, position) => {
      row[candidateIndex] = dom[s].buttons[position];
    });
    return row;
  });
  return { picker, option: found.options[0], options: found.options, slots: found.slots, buttons };
}

/** The `(slot, candidate)` pairs whose button carries `.selected`. */
export function readSelection(resolved) {
  const out = [];
  if (!resolved) return out;
  resolved.buttons.forEach((row, slot) => {
    row.forEach((button, candidate) => {
      if (button?.classList.contains('selected')) out.push({ slot, candidate });
    });
  });
  return out;
}

/**
 * Click the picker's own button for one candidate (which toggles it, exactly
 * as a click in the list would). False — and no click — when the button is
 * gone, disabled, or no longer reads as that candidate.
 */
export function selectCandidate(resolved, slotIndex, candidateIndex) {
  const button = resolved?.buttons?.[slotIndex]?.[candidateIndex];
  const candidate = resolved?.slots?.[slotIndex]?.candidates?.[candidateIndex];
  if (!button || !candidate || !button.isConnected || button.disabled) return false;
  if (text(button) !== norm(candidate.label)) return false;
  button.click();
  return true;
}

/** The picker's own "Confirm (n/max)" button: `{ button, enabled, filled, max }`, or null. */
export function confirmState(host) {
  const button = findPicker(host)?.querySelector('.picker-actions button.confirm') ?? null;
  if (!button) return null;
  const m = /\((\d+)\s*\/\s*(\d+)\)/.exec(text(button));
  return { button, enabled: !button.disabled, filled: m ? Number(m[1]) : null, max: m ? Number(m[2]) : null };
}

/**
 * Click the picker's Confirm when the announcement is complete: the button is
 * enabled (every slot within its range) AND nothing more could be added. An
 * "up to N" picker that is merely confirmable is left open. Returns whether it
 * clicked.
 */
export function confirmIfComplete(host) {
  const st = confirmState(host);
  if (!st || !st.enabled || st.filled === null || st.filled !== st.max) return false;
  st.button.click();
  return true;
}

/**
 * stackTargets — what the stack is already aimed at (`scutemob-268`, play surface v3 C3;
 * `docs/play-surface-v3-brief.md` §1 rule 6 "targeted = red outline").
 *
 * The seat view gives each stack item `target_refs`, index-aligned with its text
 * `targets`: `{ kind: 'object', id }`, `{ kind: 'player', name }` or
 * `{ kind: 'stack_object', id }` (the `StackItemView.id` of another item). An object
 * ref whose `id` is `null` is a target the server withheld from this seat
 * (Invariant 7): it is counted nowhere, drawn nowhere, and nothing is inferred from
 * its position or its text.
 *
 * Two things are derived, and both are lookups of ids the view sent:
 *
 *   - the **marks**: `objects` (object ids — `CardFace` outlines every drawn copy,
 *     `StackBox` the stack item whose `source_object_id` it is), `seats` (player
 *     names — the seat containers ring the name), `stackItems` (stack item ids);
 *   - the **hover**: the id of the stack item under the pointer (or focused), for
 *     which `Arrows.svelte` draws one red arrow per target that is on screen.
 *
 * `readStackTargets` is pure; `stackTargets` is the `$state` the view draws from,
 * written by `TableV3` whenever the stack changes.
 */

const EMPTY = new Set();

/**
 * `{ objects: Set<number>, seats: Set<string>, stackItems: Set<number>, byItem: Map<id, ref[]> }`
 * for a wire stack (`state.zones.stack`). `byItem` keeps each item's usable refs
 * in the view's order, normalised to `{ kind, id }` / `{ kind: 'player', name }`.
 */
export function readStackTargets(stack) {
  const objects = new Set();
  const seats = new Set();
  const stackItems = new Set();
  const byItem = new Map();
  for (const item of stack ?? []) {
    const refs = [];
    for (const ref of item?.target_refs ?? []) {
      if (ref?.kind === 'object') {
        // `id: null` = withheld. Not a target this seat may place.
        if (ref.id === null || ref.id === undefined || !Number.isFinite(Number(ref.id))) continue;
        objects.add(Number(ref.id));
        refs.push({ kind: 'object', id: Number(ref.id) });
      } else if (ref?.kind === 'player') {
        if (typeof ref.name !== 'string' || ref.name === '') continue;
        seats.add(ref.name);
        refs.push({ kind: 'player', name: ref.name });
      } else if (ref?.kind === 'stack_object') {
        if (ref.id === null || ref.id === undefined || !Number.isFinite(Number(ref.id))) continue;
        stackItems.add(Number(ref.id));
        refs.push({ kind: 'stack_object', id: Number(ref.id) });
      }
    }
    byItem.set(Number(item?.id), refs);
  }
  return { objects, seats, stackItems, byItem };
}

/**
 * What the view draws from.
 *   objects / seats / stackItems — see `readStackTargets`
 *   byItem — the refs of each stack item, for the hover arrows
 * `stackHover.id` — the stack item id whose arrows are shown, or null
 */
// Sets and Maps are not proxied by `$state`: each property is replaced whole (as `targeting` does).
export const stackTargets = $state({ objects: EMPTY, seats: EMPTY, stackItems: EMPTY, byItem: new Map() });
export const stackHover = $state({ id: null });

const sameSet = (a, b) => a.size === b.size && [...a].every((v) => b.has(v));

/** Fold a new stack in. Sets are replaced only when they changed, so an unchanged stack re-renders nothing. */
export function syncStackTargets(stack) {
  const next = readStackTargets(stack);
  const moved =
    !sameSet(next.objects, stackTargets.objects) || !sameSet(next.seats, stackTargets.seats) || !sameSet(next.stackItems, stackTargets.stackItems);
  // `byItem` always follows the stack (an item with no refs still has an entry).
  const sameItems =
    !moved &&
    next.byItem.size === stackTargets.byItem.size &&
    [...next.byItem].every(([id, refs]) => JSON.stringify(stackTargets.byItem.get(id)) === JSON.stringify(refs));
  if (sameItems) return;
  stackTargets.objects = next.objects;
  stackTargets.seats = next.seats;
  stackTargets.stackItems = next.stackItems;
  stackTargets.byItem = next.byItem;
  // An item that left the stack takes its arrows with it.
  if (stackHover.id !== null && !next.byItem.has(stackHover.id)) stackHover.id = null;
}

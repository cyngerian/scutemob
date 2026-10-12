/**
 * zones — one seat's piles out of the seat view (`docs/play-surface-v3-brief.md` §1 rule 4).
 *
 * Pure functions: `ZoneBox` (the 2×2 box on a centre board) and `ZoneStrip`
 * (the 1×5 strip on a side cell) draw the same five piles from here, and each
 * pile carries what its popover lists. Nothing is filtered (Invariant 7): a
 * hidden hand entry stays in the list and is drawn as a back.
 *
 * Exile is ONE list shared by all players until the server gives a per-player
 * one (brief §5, phase C3), so that pile is labelled `EX*` and says so.
 */

export const EXILE_NOTE = 'shared by all players (per-player exile is a later server change)';

const BACK = { hidden: true };

/** The five piles of `name`: `{ cmd, hand, lib, gy, ex }`, each `{ label, top, count, cz, title, note, cards }`. */
export function seatPiles(name, player, zones) {
  const command = zones?.command_zone?.[name] ?? [];
  const hand = zones?.hand?.[name] ?? [];
  const graveyard = zones?.graveyard?.[name] ?? [];
  const exile = zones?.exile ?? [];
  const handSize = player?.hand_size ?? hand.length;
  const librarySize = player?.library_size ?? 0;
  const revealed = hand.filter((c) => !c.hidden).length;
  return {
    cmd: {
      label: 'CMD',
      top: command[0] ?? null,
      count: null,
      cz: true,
      title: `${name}'s command zone`,
      note: command.length ? '' : 'commander is not in the command zone',
      cards: command,
    },
    hand: {
      label: 'HAND',
      top: handSize > 0 ? BACK : null,
      count: handSize,
      cz: false,
      title: `${name}'s hand`,
      note: revealed ? `${revealed} revealed` : 'all hidden',
      cards: hand,
    },
    lib: {
      label: 'LIB',
      top: librarySize > 0 ? BACK : null,
      count: librarySize,
      cz: false,
      title: `${name}'s library`,
      note: `hidden — ${librarySize} cards`,
      cards: [],
    },
    gy: {
      label: 'GY',
      top: graveyard[0] ?? null,
      count: graveyard.length,
      cz: false,
      title: `${name}'s graveyard`,
      note: '',
      cards: graveyard,
    },
    ex: {
      label: 'EX*',
      top: exile[0] ?? null,
      count: exile.length,
      cz: false,
      title: 'Exile',
      note: EXILE_NOTE,
      cards: exile,
    },
  };
}

/**
 * Where a peeked cell goes (rule 14): double the cell's size, capped at half
 * the viewport's width and its height less a margin, anchored on the cell's
 * own corner (its right edge for a right-hand cell) and kept on screen.
 * `rect` is the cell's `getBoundingClientRect()`.
 */
export function peekStyle(rect) {
  if (!rect) return { style: '', fromRight: false };
  const fromRight = rect.left > innerWidth / 2;
  const w = Math.min(rect.width * 2, innerWidth * 0.5);
  const h = Math.min(rect.height * 2, innerHeight - 16);
  const top = Math.max(8, Math.min(rect.top, innerHeight - h - 8));
  const right = rect.right ?? rect.left + rect.width;
  const left = Math.max(8, Math.min(fromRight ? right - w : rect.left, innerWidth - w - 8));
  return { style: `width: ${w}px; height: ${h}px; top: ${top}px; left: ${left}px;`, fromRight };
}

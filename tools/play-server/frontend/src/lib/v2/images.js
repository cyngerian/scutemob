/**
 * images.js — where a card's picture comes from (brief §4).
 *
 * Scryfall's named-image endpoint, `version=small` (146×204) for the board and
 * hand; the hover tooltip in `$viewer/cardTooltip.js` fetches `normal` of the
 * same name. The browser caches per URL, so a four-seat game is ~60 fetches.
 * A cached proxy inside `play-server` is the recorded follow-up (brief §8.4);
 * this is the one place the URL is built, so that swap is one line.
 */
export function scryfallSmallUrl(name) {
  if (!name) return null;
  return `https://api.scryfall.com/cards/named?exact=${encodeURIComponent(name)}&format=image&version=small`;
}

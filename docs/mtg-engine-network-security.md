# MTG Commander Rules Engine: Network Security Enhancements

## Purpose

This document defines a three-tier network security strategy that eliminates the need for
a trusted host in multiplayer games. It is designed to integrate with the existing architecture
(`mtg-engine-architecture.md`), roadmap (`mtg-engine-roadmap.md`), and CLAUDE.md.

The current architecture specifies an authoritative host model where one player's machine
runs the engine and all other players trust it. This document replaces that model with a
**distributed verification** approach where no single player has privileged access to game
state, hidden information is cryptographically protected, and cheating is detectable by
all participants.

---

## Threat Model

What we're protecting against in a Commander game:

| Threat | Description | Impact |
|--------|-------------|--------|
| Host sees hidden info | Host reads opponents' hands, library order, face-down cards | Unfair advantage — knows what to play around |
| Host modifies rules | Host runs modified engine that favors them | Game outcome is illegitimate |
| Player fabricates cards | Player claims to play a card not in their hand | Game integrity violated |
| State tampering | A player modifies their local state (life total, counters, etc.) | Game integrity violated |
| Eavesdropping | Third party intercepts game traffic | Privacy violation, information leak |
| Player impersonation | Someone joins pretending to be a different player | Wrong person in the game |

The three tiers of security enhancements address these threats progressively:

| Tier | Feature | Threats Mitigated |
|------|---------|-------------------|
| 1 | Deterministic state hashing | State tampering, modified rules (detectable) |
| 2 | Distributed verification (all peers run engine) | Host modifies rules, state tampering |
| 3 | Mental Poker (cryptographic card dealing) | Host sees hidden info, player fabricates cards |

---

## Tier 1: Deterministic State Hashing

### What It Is

Every `GameState` produces a deterministic hash. After each command is processed, all peers
compare hashes. If hashes disagree, something has gone wrong — either a bug (non-determinism)
or tampering.

### Why It Must Be Early

State hashing is an **invariant**, not a feature. If the engine produces different hashes for
the same command sequence on different machines, you have non-determinism — and non-determinism
means the distributed verification model (Tier 2) cannot work. Catching non-determinism early
(during M2-M3) is dramatically cheaper than discovering it during M10 when networking is built.

Common sources of non-determinism in Rust:
- `HashMap` iteration order (different across program runs unless using a fixed hasher)
- Floating-point arithmetic (shouldn't apply to MTG, but be vigilant)
- Timestamp or clock-based values leaking into state
- Random number generation without shared seeded RNG

### Implementation

#### Canonical State Hash

The hash covers all public game state. Hidden information (hand contents, library order) is
hashed separately so it can be verified by the player who owns it without revealing it to others.

```rust
use std::hash::{Hash, Hasher};
use std::collections::BTreeMap;

/// GameState hashing strategy:
/// - Public state hash: verifiable by all peers
/// - Per-player private hash: verifiable only by that player
/// - Combined hash: covers everything (for single-machine testing)

impl GameState {
    /// Hash of all publicly visible game state.
    /// All peers must agree on this after every command.
    pub fn public_state_hash(&self) -> u64 {
        let mut hasher = deterministic_hasher();
        // Turn state
        self.turn.hash(&mut hasher);
        self.turn_number.hash(&mut hasher);
        self.timestamp_counter.hash(&mut hasher);
        // Player public state (life, counters, commander damage — NOT hand contents)
        for (id, player) in self.players.iter_sorted() {
            id.hash(&mut hasher);
            player.life_total.hash(&mut hasher);
            player.commander_tax.hash(&mut hasher);
            player.commander_damage_received.hash(&mut hasher);
            player.hand_size().hash(&mut hasher); // size, not contents
        }
        // Public zones: battlefield, stack, graveyard, exile, command zone
        self.hash_public_zones(&mut hasher);
        // Continuous effects, pending triggers
        self.continuous_effects.hash(&mut hasher);
        self.pending_triggers.hash(&mut hasher);
        // Combat state
        self.combat.hash(&mut hasher);
        hasher.finish()
    }

    /// Hash of a specific player's private information.
    /// Only that player can verify this.
    pub fn private_state_hash(&self, player: PlayerId) -> u64 {
        let mut hasher = deterministic_hasher();
        // Hand contents (specific cards, not just count)
        self.zones.hand(player).hash(&mut hasher);
        // Library order
        self.zones.library(player).hash(&mut hasher);
        // Face-down cards this player controls (their identity)
        self.face_down_identities(player).hash(&mut hasher);
        hasher.finish()
    }
}
```

#### Critical Implementation Detail: Deterministic Iteration

`im::HashMap` does not guarantee iteration order across different program runs. For hashing,
you must iterate in a deterministic order. Options:

1. **Use `im::OrdMap` instead of `im::HashMap`** for all state that gets hashed. `OrdMap` is
   a B-tree map with deterministic sorted iteration. This has performance implications
   (O(log n) vs O(1) for lookups) but game state maps are small enough that it rarely matters.

2. **Collect keys, sort, then iterate** when hashing. Keeps `HashMap` for runtime performance
   and pays the sort cost only during hashing.

3. **Use a fixed hasher** for `im::HashMap` (e.g., `ahash` with a fixed seed). This makes
   iteration order deterministic for the same data, though it's implementation-dependent
   and could break across `im` versions.

**Recommendation**: Use `im::OrdMap` for any maps that are part of game state. The performance
difference is negligible for the map sizes in MTG (max ~hundreds of entries), and deterministic
iteration is guaranteed by the data structure rather than by convention.

#### Property Test

This test should run from M2 onward, on every commit:

```rust
#[test]
fn test_state_hash_determinism() {
    // Create two independent engine instances
    let seed = 12345u64;
    let commands = generate_random_command_sequence(seed, 100);

    let mut state_a = GameState::new_commander_game(seed);
    let mut state_b = GameState::new_commander_game(seed);

    for command in &commands {
        state_a = engine::process(state_a, command.clone()).unwrap();
        state_b = engine::process(state_b, command.clone()).unwrap();

        assert_eq!(
            state_a.public_state_hash(),
            state_b.public_state_hash(),
            "Public state hash diverged after command: {:?}",
            command
        );
    }
}
```

This test is cheap, catches non-determinism immediately, and validates a prerequisite for
all higher-tier security features.

### Integration Points

| Where | What | When |
|-------|------|------|
| `GameState` struct | Add `public_state_hash()` and `private_state_hash()` methods | M2-M3 |
| `im::HashMap` → `im::OrdMap` | Use OrdMap for all game state maps | M1 (data model) or M2 |
| Property tests | Dual-instance hash comparison test | M2 onward, every commit |
| Network protocol | Include `public_state_hash` in every event broadcast | M10 |
| Dispute detection | Hash mismatch triggers dispute flow | M10 |

---

## Tier 2: Distributed Verification

### What It Is

Instead of one authoritative host running the engine and broadcasting results, **all peers
run the engine independently**. Every player processes every command locally and verifies
the result. The "host" becomes a lightweight **coordinator** that manages protocol sequencing
(whose turn it is, whose priority it is) but does not have privileged access to game state.

### Network Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Alice      │     │   Bob        │     │   Carol      │     │   Dave       │
│ ┌─────────┐ │     │ ┌─────────┐ │     │ ┌─────────┐ │     │ ┌─────────┐ │
│ │ Engine  │ │     │ │ Engine  │ │     │ │ Engine  │ │     │ │ Engine  │ │
│ │ (full   │ │     │ │ (full   │ │     │ │ (full   │ │     │ │ (full   │ │
│ │ public  │ │     │ │ public  │ │     │ │ public  │ │     │ │ public  │ │
│ │ + own   │ │     │ │ + own   │ │     │ │ + own   │ │     │ │ + own   │ │
│ │ private)│ │     │ │ private)│ │     │ │ private)│ │     │ │ private)│ │
│ └─────────┘ │     │ └─────────┘ │     │ └─────────┘ │     │ └─────────┘ │
│ Coordinator │     │             │     │             │     │             │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │                   │
       └───────────────────┴───────────────────┴───────────────────┘
                          Peer-to-peer mesh
                    (commands, hashes, proofs)
```

### Coordinator vs. Authoritative Host

The coordinator role is much lighter than an authoritative host:

| Responsibility | Authoritative Host | Coordinator |
|---------------|-------------------|-------------|
| Runs the engine | Only the host | All peers |
| Validates commands | Host validates, rejects illegal | All peers validate independently |
| Knows full game state | Yes — sees all hands, libraries | No — only knows own private state |
| Sequences protocol | Yes | Yes (only remaining responsibility) |
| Resolves disputes | Host is always right | Majority vote among peers |
| Single point of failure | Yes — host crash kills game | Coordinator role can migrate |

The coordinator's job is limited to:
1. Announcing whose priority it is
2. Receiving the acting player's command
3. Broadcasting the command to all peers
4. Collecting hash confirmations from all peers
5. Announcing consensus (all hashes match) or flagging dispute

Any peer can be the coordinator. If the coordinator disconnects, another peer takes over.
This is simpler than host migration in the authoritative model because the coordinator doesn't
hold any special state — it's just traffic directing.

### Protocol Flow

```
Normal command flow:
1. Coordinator: "Bob has priority"
2. Bob → all peers: Command::CastSpell { ... }
3. Each peer: processes command through local engine
4. Each peer → coordinator: StateHashConfirmation { public_hash: 0xABCD }
5. Coordinator: all hashes match → "Command accepted"
   OR: hashes disagree → "DISPUTE — hashes: Alice=0xABCD, Bob=0xABCD, Carol=0x1234, Dave=0xABCD"

Dispute resolution:
1. Majority hash wins (3 vs 1 in the example above)
2. Minority peer is flagged (likely bug or tampering)
3. Minority peer receives a full public state sync from majority
4. If disputes persist, minority peer is kicked
```

### Hidden Information in the Distributed Model

With distributed verification alone (no Mental Poker), there's a question: how do players
draw cards if no central authority knows the library order?

**Temporary solution (Tier 2 only):** Use a **shared seeded RNG**. At game start, all peers
agree on a random seed (each player contributes entropy, combined via XOR or hash). The RNG
determines library order, random choices, etc. All peers use the same seed, so all engines
produce the same "random" results deterministically.

The weakness: if you know the seed, you can compute future draws. In the distributed model,
ALL players know the seed (they have to, for determinism). So every player can compute
everyone's library order. This is strictly worse than the authoritative host model for library
secrecy.

This is why Tier 3 (Mental Poker) is needed. Tier 2 alone gives you tamper detection and
distributed rules enforcement, but not hidden information protection. The two tiers complement
each other:
- Tier 2 ensures nobody cheats on rules or public state
- Tier 3 ensures nobody sees hidden information

### Coordinator Election and Migration

If the coordinator disconnects, the remaining peers need to elect a new one. A simple protocol:

1. Each peer has a **peer ID** (assigned at game creation, based on turn order)
2. The coordinator is always the lowest-numbered connected peer
3. When a peer detects the coordinator is gone (heartbeat timeout), the next-lowest peer
   automatically becomes coordinator
4. The new coordinator announces itself and resumes from the last confirmed state hash

This works because the coordinator has no special state — any peer can take over immediately.

### What Changes from the Authoritative Host Model

| Component | Authoritative Host (original) | Distributed Verification |
|-----------|------------------------------|-------------------------|
| Network topology | Star (host ↔ clients) | Mesh (all peers ↔ all peers) |
| Engine instances | 1 (on host) | N (one per peer) |
| State projection | Host filters hidden info per client | Not needed — each peer has own private state |
| Command validation | Host validates | All peers validate independently |
| Dispute resolution | Host is always right | Majority vote |
| Reconnection | Full state dump from host | Public state from majority + private state reconstruction |
| Coordinator failure | Game dies | Coordinator migrates to next peer |

### Implementation Scope for M10

The network crate architecture changes from:
```
network/
├── host/       ← authoritative game host
├── client/     ← client connection, state mirror
├── protocol/
└── lobby/
```

To:
```
network/
├── peer/           ← each peer's networking logic
├── coordinator/    ← coordinator role (lightweight)
├── mesh/           ← peer-to-peer connection management
├── consensus/      ← hash comparison, dispute resolution
├── protocol/       ← message types, serialization
└── lobby/          ← game creation, peer discovery
```

The engine crate does not change. The Tauri IPC bridge changes minimally (it talks to a
local engine instance either way).

---

## Tier 3: Mental Poker (Cryptographic Card Dealing)

### What It Is

Mental Poker is a cryptographic protocol that allows players to shuffle, deal, and draw cards
such that:
- No player knows the deck order after shuffling
- Only the drawing player sees their drawn card
- A player cannot claim to have drawn a card they didn't
- Revealed cards are provably legitimate

This eliminates the hidden information problem that Tier 2's shared RNG cannot solve.

### The Protocol (Simplified for MTG)

MTG uses Mental Poker for three operations: **deck initialization** (shuffling at game start),
**drawing** (revealing a card to one player), and **searching** (revealing library contents
to one player for a tutor effect).

#### Cryptographic Primitives

Mental Poker uses **commutative encryption** — an encryption scheme where:
```
Encrypt_A(Encrypt_B(card)) = Encrypt_B(Encrypt_A(card))
```

This means encryption can be applied and removed in any order. ElGamal encryption over
elliptic curves provides this property. Rust crate: `curve25519-dalek`.

Each player generates a **keypair** at game start. Their encryption key is used to "lock"
cards, and their decryption key is used to "unlock" them.

#### Deck Initialization (Joint Shuffle)

```
1. Start with a canonical deck representation:
   [Card_1, Card_2, ..., Card_100] (the 100 cards in the Commander deck)

2. Alice encrypts each card with her key:
   [E_A(Card_1), E_A(Card_2), ..., E_A(Card_100)]
   Then shuffles the encrypted list randomly.

3. Alice sends the shuffled, encrypted list to Bob.

4. Bob encrypts each (already encrypted) card with his key:
   [E_B(E_A(Card_?)), E_B(E_A(Card_?)), ...]
   Then shuffles again.

5. Repeat for Carol, then Dave.

6. Result: each card is encrypted under all 4 keys, shuffled 4 times.
   The final order is the library order.
   No single player knows the mapping between positions and cards.
```

Each player must provide a **zero-knowledge proof of shuffle** — proving they permuted the
list without revealing which permutation they used. This prevents a player from "shuffling"
in a way that puts known cards in known positions.

#### Drawing a Card

```
1. The top card of the library is: E_D(E_C(E_B(E_A(Card_?))))
   (encrypted under all 4 keys)

2. Alice wants to draw. She asks Bob, Carol, and Dave to each
   remove their encryption layer from the top card.

3. Bob decrypts his layer: E_D(E_C(E_A(Card_?)))  → sends to Alice
4. Carol decrypts her layer: E_D(E_A(Card_?))      → sends to Alice
5. Dave decrypts his layer: E_A(Card_?)             → sends to Alice

6. Alice decrypts her own layer: Card_?             → Alice now sees the card

7. Alice announces she drew a card (not which one).
   She stores a commitment to the card's identity for later proof.
```

Bob, Carol, and Dave each only see a partially-decrypted blob — they can't determine the
card without Alice's key.

#### Library Search (Tutoring)

Searching is more complex because the player needs to see all cards in the library, choose
one, and prove the choice is valid without revealing the rest.

```
1. Alice casts Demonic Tutor (search library for a card).

2. For EACH card in Alice's library, Bob, Carol, and Dave remove
   their encryption layers and send the partially decrypted cards to Alice.

3. Alice removes her own encryption from each card.
   Alice now sees her entire library.

4. Alice chooses a card (say, position 17).

5. Alice announces "I found a card" and provides a commitment to position 17.

6. The card at position 17 is removed from the encrypted library.
   The remaining cards are RE-ENCRYPTED and RE-SHUFFLED
   (another joint shuffle, to hide the new order since Alice saw it).

7. Alice puts the chosen card into her hand.
```

The re-encryption and re-shuffle after a search is expensive (another full shuffle protocol)
but necessary — Alice saw the library order, so it must be randomized again.

### Performance Characteristics

| Operation | Network Round Trips | Computation (per player) | Latency Estimate |
|-----------|-------------------|------------------------|-----------------|
| Deck init (shuffle) | 4 (one per player) | ~100 EC operations | ~200-500ms total |
| Draw a card | 1 (parallel decryption) | ~4 EC operations | ~50-100ms |
| Search library | 1 + 4 (reveal + reshuffle) | ~100 EC operations | ~500ms-1s |
| Reveal from hand | 1 (commitment reveal) | ~1 EC operation | ~10ms |

These latencies are acceptable for a turn-based game. Drawing a card takes <100ms, which is
imperceptible. Library searching takes up to 1 second, which is noticeable but tolerable
(and the player is making a decision during this time anyway).

### What Mental Poker Does NOT Cover

Some MTG effects create information asymmetries that Mental Poker doesn't natively handle:

- **"Look at target player's hand"** — requires selectively revealing hand contents to one
  player. Solvable: the target player decrypts their hand cards and sends them to the viewer.
  Other players don't see this exchange.

- **"Reveal the top card of your library"** — requires decrypting one card publicly. All
  players remove their encryption from the top card, making it visible to everyone.

- **Scry** — player looks at top N cards and reorders them. The player sees the cards
  (via draw protocol), chooses an order, and the cards are re-encrypted in the new order.
  The re-ordering is private but the fact that scrying occurred is public.

- **Face-down cards (Morph)** — the player knows the identity, opponents don't. This maps
  naturally to Mental Poker — the card is in a public zone but its identity is still
  encrypted under the controller's key. Turning it face-up is a reveal operation.

Each of these is solvable within the protocol framework but requires specific handling in
the engine integration.

### Rust Implementation Sketch

```rust
// Crate: network/src/crypto/mental_poker.rs

use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;

/// A player's keypair for Mental Poker operations.
pub struct PlayerCryptoKeys {
    pub encrypt_key: Scalar,      // private
    pub decrypt_key: Scalar,      // private (inverse of encrypt_key)
    pub public_key: RistrettoPoint, // shared with all peers
}

/// An encrypted card — a point on the elliptic curve.
/// The underlying card identity is mapped to a curve point.
pub struct EncryptedCard(RistrettoPoint);

/// Encrypt a card with a player's key.
pub fn encrypt(card: &EncryptedCard, key: &Scalar) -> EncryptedCard {
    EncryptedCard(card.0 * key)
}

/// Decrypt a card by removing one player's encryption layer.
pub fn decrypt(card: &EncryptedCard, key: &Scalar) -> EncryptedCard {
    let inverse = key.invert();
    EncryptedCard(card.0 * inverse)
}

/// A shuffled, encrypted deck.
pub struct EncryptedDeck {
    cards: Vec<EncryptedCard>,
    /// Proof that the shuffle was performed correctly (ZK proof).
    shuffle_proof: ShuffleProof,
}

/// The joint shuffle protocol for deck initialization.
pub struct ShuffleProtocol {
    peers: Vec<PlayerId>,
    current_phase: ShufflePhase,
    deck_state: Vec<EncryptedCard>,
}

impl ShuffleProtocol {
    /// Start a new shuffle with a plaintext deck.
    pub fn new(deck: Vec<CardId>, peers: Vec<PlayerId>) -> Self { ... }

    /// Process one player's shuffle contribution.
    /// Verifies the shuffle proof before accepting.
    pub fn apply_shuffle(&mut self, from: PlayerId, shuffled: EncryptedDeck) -> Result<(), ShuffleError> { ... }

    /// Check if all players have shuffled.
    pub fn is_complete(&self) -> bool { ... }

    /// Get the final encrypted deck.
    pub fn finalize(self) -> Result<Vec<EncryptedCard>, ShuffleError> { ... }
}

/// The draw protocol — reveal a card to one player.
pub struct DrawProtocol {
    card: EncryptedCard,
    decryptions_received: HashMap<PlayerId, EncryptedCard>,
    target_player: PlayerId,
}

impl DrawProtocol {
    /// Start a draw — the card to be drawn and who draws it.
    pub fn new(card: EncryptedCard, target: PlayerId) -> Self { ... }

    /// Receive a partial decryption from one peer.
    pub fn receive_decryption(&mut self, from: PlayerId, partial: EncryptedCard) -> Result<(), DrawError> { ... }

    /// Check if all decryptions are in.
    pub fn is_complete(&self) -> bool { ... }

    /// Finalize — only the target player calls this to see the card.
    pub fn finalize(self, own_key: &Scalar) -> Result<CardId, DrawError> { ... }
}
```

### Integration With the Engine

The engine itself does not know about encryption. The integration layer sits between the
engine and the network:

```
Player wants to draw a card
         │
         ▼
Network layer: initiate DrawProtocol for top card of library
         │
         ▼
Other peers: each sends partial decryption
         │
         ▼
Drawing player: finalizes, sees the card
         │
         ▼
Engine: player.draw(card_id) — engine receives the plaintext card
         │
         ▼
All peers: engine processes "player drew a card" (public event)
           Each peer's engine updates hand_size for that player
           Only the drawing player's engine knows the card identity
```

The engine's `Command::DrawCard` operates on plaintext card IDs. The Mental Poker protocol
is a pre-processing step that determines WHICH card ID the player draws. This keeps the
engine clean — no cryptographic dependencies in the engine crate.

---

## Integration With Existing Game State Model

### Dual State: Public vs. Private

With distributed verification, each peer maintains:

```rust
pub struct PeerGameState {
    /// Full public game state — battlefield, stack, graveyard, exile, etc.
    /// Verified by all peers via hash comparison.
    pub public_state: GameState,

    /// This peer's private knowledge.
    /// Not shared with other peers.
    pub private_state: PrivateState,
}

pub struct PrivateState {
    /// Cards in this player's hand (plaintext — only this player knows).
    pub hand: Vec<CardId>,

    /// This player's library order (known only after Mental Poker draw/search).
    /// Before a card is drawn, its identity is unknown even to the owning player.
    pub known_library_cards: HashMap<usize, CardId>,

    /// Commitments for cards in hand (for later proof of legitimate play).
    pub hand_commitments: Vec<CardCommitment>,

    /// Encryption keys for Mental Poker.
    pub crypto_keys: PlayerCryptoKeys,
}
```

The engine crate's `GameState` is the public state. The `PrivateState` wraps it with
player-specific knowledge. The engine doesn't know about `PrivateState` — it's managed
by the network layer.

### RNG Replacement

With Mental Poker, the shared seeded RNG is no longer used for library shuffling or card
drawing. It's still used for other random events (coin flips, random selection from a set
of options) where the result should be public and verifiable.

For these public random events, the peers use a **commit-reveal** scheme:
1. Each peer generates a random value and broadcasts a hash (commitment)
2. After all commitments are received, each peer reveals their value
3. The final random result is the XOR (or hash combination) of all revealed values
4. No single peer can bias the outcome

---

## When to Implement Each Tier

### Tier 1: Deterministic State Hashing

**When**: Immediately (during M2-M3). Retroactively easy to add.

**Tasks**:
- [ ] Evaluate `im::OrdMap` vs `im::HashMap` for game state maps — switch to OrdMap if
      hash determinism is easier to guarantee
- [ ] Implement `public_state_hash()` on `GameState`
- [ ] Implement `private_state_hash()` on `GameState`
- [ ] Add dual-instance hash comparison property test
- [ ] Run property test on every commit in CI

**Impact on existing milestones**: Minimal. Adds one property test and possibly changes
the map type in the state model. No architectural changes.

**Estimated effort**: 1-2 days.

### Tier 2: Distributed Verification

**When**: M10 (replaces the authoritative host model, not in addition to it).

**Tasks**:
- [ ] Implement peer-to-peer mesh networking (WebSocket mesh or WebRTC)
- [ ] Implement coordinator role (protocol sequencing, command broadcasting)
- [ ] All peers run engine independently on received commands
- [ ] Hash comparison after every command
- [ ] Dispute detection and majority-vote resolution
- [ ] Coordinator election and migration on disconnect
- [ ] Reconnection protocol (public state sync from majority peers)
- [ ] Lobby system adapted for peer model (one peer creates, others join, but all are equal once game starts)

**Impact on existing milestones**: M10 is redesigned but not significantly larger. The
coordinator is simpler than the authoritative host's state projection. The mesh networking
is more complex than star topology but well-supported by existing crates.

**Estimated effort**: Same as original M10 (~3-4 weeks). Complexity shifts from state
projection to consensus and mesh management.

### Tier 3: Mental Poker

**When**: M10.5 (new milestone between M10 and M11).

**Tasks**:
- [ ] Implement commutative encryption (ElGamal over Curve25519)
- [ ] Implement joint shuffle protocol with ZK shuffle proofs
- [ ] Implement draw protocol (partial decryption from each peer)
- [ ] Implement search protocol (full library reveal + re-shuffle)
- [ ] Implement commit-reveal for public random events
- [ ] Integrate with engine's draw, search, and shuffle commands
- [ ] Handle MTG-specific operations: scry, reveal, look at hand, morph/manifest
- [ ] Implement hand commitment system (prove you had a card when you play it)
- [ ] Performance testing: verify draw latency <100ms, search latency <1s

**Impact on existing milestones**: New milestone. Does not affect M0-M9 (engine development).
Does not affect M11+ (UI) except that the UI may show a brief "shuffling..." indicator
during deck initialization.

**Estimated effort**: ~2-3 weeks. The cryptographic primitives are provided by existing
crates. The integration with the engine and networking is the main work.

### Revised Milestone Sequence (Networking Section Only)

```
M9: Commander Rules Integration (engine core complete)
 │
M10: Networking Layer (REVISED — distributed verification)  (~3-4 weeks)
 │   - Peer-to-peer mesh
 │   - Coordinator role
 │   - All peers run engine
 │   - Hash comparison consensus
 │   - Dispute detection and resolution
 │   - Coordinator migration
 │
M10.5: Mental Poker Integration (NEW)                       (~2-3 weeks)
 │   - Cryptographic deck shuffling
 │   - Draw and search protocols
 │   - Hand commitment system
 │   - MTG-specific reveal operations
 │
M11: Tauri App Shell & Basic UI
```

---

## Limitations and Honest Assessment

### What This Architecture Achieves

- **No trusted host.** No single player has privileged access to game state.
- **Tamper detection.** Any modification to public state is detected immediately.
- **Hidden information protection.** Library order and hand contents are cryptographically
  secured (with Tier 3).
- **Coordinator is lightweight.** No special state, easily migrated.
- **Engine remains pure.** All cryptographic complexity is in the network layer.

### What This Architecture Does NOT Achieve

- **Collusion resistance.** If two players share their private keys, they can decrypt
  each other's cards. In a 4-player Commander game, two colluding players can reconstruct
  significant information. This is inherent to any system without a trusted third party.

- **Guaranteed availability.** If 2+ players disconnect simultaneously, the game may not
  be recoverable (not enough peers for majority vote or decryption). This is unlikely in
  practice.

- **Protection against a modified client.** A player running a modified client could refuse
  to decrypt cards during the draw protocol, effectively stalling the game. The protocol
  handles this via timeout — if a player doesn't respond to a decryption request within N
  seconds, they're considered disconnected.

- **Perfect information hiding during searches.** When a player searches their library,
  they see the entire library contents. They then re-shuffle, but they could memorize the
  positions of cards they didn't take. The re-shuffle makes this knowledge useless (the
  order changes), but it's a theoretical information leak during the search window.

- **Concealment of play patterns.** Even with encryption, the timing and structure of
  network messages can leak information. A player who takes a long time during their
  draw step might signal they drew something important. This is a side-channel that
  exists in any networked card game and is not addressed by this architecture.

### Fallback: Trusted Host Mode

Despite all the above, the app should still support a **trusted host mode** as a fallback.
Reasons:
- Simpler to debug during development
- Lower latency (no shuffle protocol overhead)
- Works with 2 players (Mental Poker with only 2 participants is weaker cryptographically)
- Some play groups genuinely don't care about security

The mode should be selectable at game creation: "Secure mode (distributed)" vs
"Trusted host mode (one player hosts)." The engine doesn't care — it's the network layer
that switches behavior.

---

## Dependencies and Crate Recommendations

| Purpose | Crate | Notes |
|---------|-------|-------|
| Elliptic curve operations | `curve25519-dalek` | Well-audited, high performance |
| Scalar arithmetic | (included in curve25519-dalek) | |
| Zero-knowledge proofs | `merlin` (transcript) + `bulletproofs` | For shuffle proofs |
| Hashing | `blake3` or `sha2` | For commitments and state hashing |
| Serialization | `serde` + `rmp-serde` | Already in the project |
| WebSocket mesh | `tokio-tungstenite` | Already planned for M10 |
| Peer discovery | `mdns` (for LAN) | Nice to have for local games |

//! Proof-of-concept: im-rs structural sharing for game state snapshots.
//!
//! Demonstrates that cloning im-rs data structures is O(1) via structural
//! sharing, and that modifications to a clone do not affect the original.
//! This validates the immutable state approach described in the architecture doc.

use im::{HashMap, Vector};

/// Minimal stand-in for a game object, representing a permanent on the battlefield.
#[derive(Clone, Debug, PartialEq)]
struct GameObject {
    id: u64,
    name: String,
    power: i32,
    toughness: i32,
}

/// Minimal stand-in for a player's state.
#[derive(Clone, Debug, PartialEq)]
struct PlayerState {
    life_total: i32,
    hand: Vector<u64>,
}

/// Minimal stand-in for the full game state using im-rs collections.
#[derive(Clone, Debug)]
struct GameState {
    objects: HashMap<u64, GameObject>,
    players: HashMap<u64, PlayerState>,
    turn_number: u32,
}

#[test]
fn test_clone_is_independent() {
    let mut state = GameState {
        objects: HashMap::new(),
        players: HashMap::new(),
        turn_number: 1,
    };

    // Add some objects and players
    state.objects.insert(
        1,
        GameObject {
            id: 1,
            name: "Grizzly Bears".into(),
            power: 2,
            toughness: 2,
        },
    );
    state.players.insert(
        100,
        PlayerState {
            life_total: 40,
            hand: Vector::from(vec![10, 11, 12]),
        },
    );

    // Clone the state (O(1) via structural sharing)
    let snapshot = state.clone();

    // Modify the original — snapshot should be unaffected
    state.objects.insert(
        2,
        GameObject {
            id: 2,
            name: "Lightning Bolt".into(),
            power: 0,
            toughness: 0,
        },
    );
    state.players.get_mut(&100).unwrap().life_total = 37;
    state.turn_number = 2;

    // Snapshot retains original values
    assert_eq!(snapshot.objects.len(), 1);
    assert!(snapshot.objects.contains_key(&1));
    assert!(!snapshot.objects.contains_key(&2));
    assert_eq!(snapshot.players[&100].life_total, 40);
    assert_eq!(snapshot.turn_number, 1);

    // Original has new values
    assert_eq!(state.objects.len(), 2);
    assert_eq!(state.players[&100].life_total, 37);
    assert_eq!(state.turn_number, 2);
}

#[test]
fn test_structural_sharing_large_state() {
    // Build a state with many objects (simulating a complex board)
    let mut objects = HashMap::new();
    for i in 0..1000 {
        objects.insert(
            i,
            GameObject {
                id: i,
                name: format!("Permanent_{}", i),
                power: (i % 10) as i32,
                toughness: (i % 10) as i32,
            },
        );
    }

    let state = GameState {
        objects,
        players: HashMap::new(),
        turn_number: 1,
    };

    // Clone should be near-instant due to structural sharing
    let snapshot = state.clone();

    // Both have 1000 objects
    assert_eq!(state.objects.len(), 1000);
    assert_eq!(snapshot.objects.len(), 1000);

    // Modifying one object in the clone doesn't affect the original
    let mut modified = snapshot.clone();
    modified.objects.insert(
        500,
        GameObject {
            id: 500,
            name: "Modified".into(),
            power: 99,
            toughness: 99,
        },
    );

    assert_eq!(snapshot.objects[&500].name, "Permanent_500");
    assert_eq!(modified.objects[&500].name, "Modified");
}

#[test]
fn test_vector_ordering_preserved() {
    // im::Vector preserves insertion order — important for stack, library, etc.
    let mut stack: Vector<String> = Vector::new();
    stack.push_back("Lightning Bolt".into());
    stack.push_back("Counterspell".into());
    stack.push_back("Giant Growth".into());

    let snapshot = stack.clone();

    // LIFO: pop from the back
    let top = stack.pop_back().unwrap();
    assert_eq!(top, "Giant Growth");
    assert_eq!(stack.len(), 2);

    // Snapshot still has all 3
    assert_eq!(snapshot.len(), 3);
    assert_eq!(snapshot[2], "Giant Growth");
}

#[test]
fn test_snapshot_benchmark_feasibility() {
    // Rough proof that cloning a large state is cheap.
    // Not a proper benchmark, just a sanity check that it doesn't blow up.
    let mut objects = HashMap::new();
    for i in 0..500 {
        objects.insert(
            i,
            GameObject {
                id: i,
                name: format!("Card_{}", i),
                power: 1,
                toughness: 1,
            },
        );
    }

    let mut players = HashMap::new();
    for p in 0..4 {
        let mut hand = Vector::new();
        for c in 0..7 {
            hand.push_back(p * 100 + c);
        }
        players.insert(
            p,
            PlayerState {
                life_total: 40,
                hand,
            },
        );
    }

    let state = GameState {
        objects,
        players,
        turn_number: 1,
    };

    // Clone 1000 times — should be fast with structural sharing
    let mut snapshots = Vec::with_capacity(1000);
    for _ in 0..1000 {
        snapshots.push(state.clone());
    }

    assert_eq!(snapshots.len(), 1000);
    assert_eq!(snapshots[999].objects.len(), 500);
    assert_eq!(snapshots[999].players.len(), 4);
}

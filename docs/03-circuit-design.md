# Circuit design (proposal)

Status: **design only, not implemented**. Gate counts are estimates from the
per-gadget costs in [02-ragu-primer.md](02-ragu-primer.md). Confirm them with
Ragu's metrics driver once the steps compile.

## Conventions

- Board: 10×10, cell index `i = y * 10 + x`, with `x, y ∈ [0, 10)`.
- Fleet: lengths `[5, 4, 3, 3, 2]`, 17 occupied cells in total.
- `occ[i] ∈ {0, 1}`: whether cell `i` holds a ship.
- `pack(occ)`: the 100 bits packed little-endian into one Pallas element
  (`multipack`, no gates).
- Commitment: `C = Poseidon(pack(occ), salt)`, where `salt` is a random field
  element chosen by the player and kept secret. Two inputs fit in one
  permutation (~288 gates).

## Headers

| Header | Data | Elements | Meaning |
| --- | --- | --- | --- |
| `BoardHeader` | `C` | 1 | "C commits to a legal fleet." |
| `ShotHeader` | `(C, i, hit)` | 3 | "The legal board committed in C has `occ[i] = hit`." |

`HEADER_SIZE = 4` covers both, with room for a game or session id if needed.

The shot header carries the cell index `i`, not `(x, y)`. The shooter checks
`i == 10 * y + x` for the shot they asked for, natively, so the circuit needs no
range checks on `x` and `y`.

## Step 1: `BoardStep` (seed, `() × () → BoardHeader`)

Witness: one placement choice per ship, plus `salt`.

### Placement by one-hot selection

A ship of length `L` has `10 · (11 − L)` horizontal and `10 · (11 − L)`
vertical legal placements. Each placement `p` has a fixed 100-bit mask `M_p`
that can be computed at compile time.

| Ship | L | Placements |
| --- | --- | --- |
| Carrier | 5 | 120 |
| Battleship | 4 | 140 |
| Cruiser | 3 | 160 |
| Submarine | 3 | 160 |
| Destroyer | 2 | 180 |
| **Total** | | **760** |

For each ship `k`, allocate a boolean selector `s[k][p]` for each of its
placements, and constrain:

1. `s[k][p] ∈ {0, 1}`: `Boolean::alloc`.
2. `Σ_p s[k][p] = 1`: exactly one placement per ship (linear).
3. `occ[i] = Σ_k Σ_{p : M_p[i] = 1} s[k][p]` (linear, virtual wire).
4. `occ[i] ∈ {0, 1}` for all 100 cells. Because every ship contributes 0 or 1
   to each cell, this rules out overlaps.

Bounds and row wrap-around never need checking: only legal placements exist as
selectors. This replaces the Halo2 version's running-sum placement chip and
transpose chip with constant masks and linear algebra.

### Commitment

5. `C = Poseidon(pack(occ), salt)`. Output `BoardHeader(C)`.

### Budget

| Part | Gates | Constraints |
| --- | ---: | ---: |
| 760 selectors (`Boolean::alloc`) | 760 | 1,520 |
| 5 "exactly one" sums | 0 | 5 |
| 100 cell booleanity (overlap) checks | 100 | ~300 |
| `salt` alloc | ≤1 | — |
| Poseidon, 2 inputs | ~288 | (internal) |
| Framework overhead + header | small | small |
| **Estimate** | **~1,150** | **~2,000** |
| Limit at `R<13>` | 2,048 | 8,192 |

It fits in one step with ~40% headroom.

Fallback if it does not: give each ship one orientation bit and two 10-way
one-hots (or split the board step into two fused steps). Since node count
drives proving time, try one step first.

## Step 2: `ShotStep` (fuse, `BoardHeader × () → ShotHeader`)

Inputs: left child is the player's own `BoardHeader` PCD from step 1. Right
child is a unit PCD (the bootstrap proof).

Witness: `occ[0..100]` (from the board witness), `salt`, and the shot index `i`.

1. `occ[j] ∈ {0, 1}`: 100 × `Boolean::alloc`.
2. `Poseidon(pack(occ), salt) == C_left`, which binds the opening to the board
   proven legal in the child.
3. Shot one-hot `t[j]`: 100 × `Boolean::alloc`, with `Σ t[j] = 1` and
   `Σ j · t[j] = i`.
4. `hit = Σ t[j] · occ[j]`: 100 × `mul`. `hit` is automatically 0 or 1.
5. Output `ShotHeader(C, i, hit)`.

| Part | Gates |
| --- | ---: |
| occ bits | 100 |
| Poseidon re-open | ~288 |
| shot one-hot | 100 |
| hit products | 100 |
| **Estimate** | **~600** |

The fuse checks the child's board proof recursively. **A compressed
`ShotHeader` proof therefore implies board legality by itself**, so the
opponent never needs a separate board proof. The first message of a game can
still be the compressed board proof, so that `C` is fixed before any shots.

### Soundness notes

- The board is bound by `C` from the child header, not by anything the prover
  supplies in the shot step.
- The shooter must check natively that `C` equals the commitment received at
  game start, and that `i` is the cell they fired at. A valid proof for the
  wrong `C` or `i` must be rejected at the protocol layer.
- Repeated shots are harmless here: each proof stands alone. The phase-2
  state chain below handles uniqueness.

## Phase 2 (later): game-state chain

The user-facing PoC trusts clients to tally hits. Because every hit is a
verified proof, the **shooter can count opponent hits from verified
`ShotHeader`s alone**. Win detection is already locally verifiable. A single
aggregate proof is only needed for a third party (a contract, or the state
channel the Plonky2 version prototyped).

A sketch for that aggregate:

```
GameHeader = (C, shots_bitmap, hits_taken, [opponent C / session id])

Init:   BoardHeader × ()  → GameHeader(C, 0, 0)
Answer: GameHeader × ()   → GameHeader(C, shots' = shots | t, hits' = hits + hit)
        with   t · shots = 0  (no repeat shots)
```

`Answer` costs roughly the shot step plus about 200 gates for unpacking and
checking the shots bitmap, around 900 gates in total, which still fits in one
step. The defender keeps the uncompressed chain locally and sends a compressed
snapshot each turn. A final snapshot with `hits_taken = 17` is a succinct proof
of loss.

# Prior art: earlier BattleZips circuits

Every BattleZips version splits the game into the same two statements:

- **Board**: "I know a fleet placement that is legal, and `C` is a commitment to it."
- **Shot**: "The board committed in `C` does (or does not) have a ship at `(x, y)`."

What changes between versions is how the board is encoded, which commitment is
used, and how placement legality is enforced.

## BattleZips-Halo2 (zcash/halo2 0.2, Pallas)

Source: <https://github.com/BattleZips/BattleZips-Halo2> (`src/chips`, `src/circuits`).

### Encoding

- Each ship is a **100-bit field element** with its occupied cells set, at index
  `y * 10 + x`.
- The fleet is 10 inputs, a horizontal and vertical slot for each ship:
  `[H5, V5, H4, V4, H3a, V3a, H3b, V3b, H2, V2]`. One slot in each pair must be 0.
- The board state is the OR (in practice the sum) of all ship bitmaps: a single
  100-bit field element.
- A shot is also a 100-bit value with exactly one bit set.

### Board circuit

1. For each H/V pair, constrain that at least one is zero.
2. Decompose all 10 ship values into bits (`bitify` chip).
3. `PlacementChip` checks each ship with running sums over its bits: exactly
   `L` bits set, and they form one full window of `L` consecutive bits
   (horizontal) or bits spaced 10 apart (vertical). Placing a ship this way also
   blocks wrapping across row edges.
4. `TransposeChip` sums the ship bitmaps cell by cell into one board bitmap.
   Each cell sum must be boolean, which rules out overlaps.
5. Recompose the board bits into a field element and compute a **Pedersen
   commitment** `v * board + r * trapdoor` on Pallas, using Orchard-style fixed-base
   scalar multiplication. The commitment point `(x, y)` is the public output.

### Shot circuit

Public inputs: board commitment, shot bitmap, hit flag. Private: board state
and trapdoor.

1. Constrain `hit` to be boolean.
2. Decompose board and shot into 100 bits each.
3. Running sums: the shot has exactly one bit set, and `hit` equals the number
   of cells where both the shot bit and the board bit are set.
4. Recompute the Pedersen commitment and match it to the public one.

### Takeaways

- ~11k lines of Rust. Most of it is custom chips, layouts, and fixed-base tables.
  The halo2 PLONKish API was the main cost, not the game logic.
- The README notes that recursion and state channels "eluded us" in halo2, which
  led to the Plonky2 PCD prototype. Ragu is a PCD framework from the start, so
  that gap should close.
- Bitmap encoding plus an "each cell is boolean" overlap check is the right idea
  and carries over directly.

## BattleZips-Noir

Source: <https://github.com/BattleZips/BattleZips-Noir> (`circuits/board`, `circuits/shot`).

### Encoding

`ships: [Field; 15]`, which is `(x, y, z)` for each of the 5 ships. `z` is the
orientation (1 = vertical). Ship lengths are fixed at `[5, 4, 3, 3, 2]`.

### Board circuit

- Range checks: `z <= 1`, and the ship fits on the board in its orientation.
- Collision check: expand every ship into cell indices, then compare each cell
  against every earlier cell (O(17²) equality checks).
- `pedersen(ships) == hash` (public).

### Shot circuit

- `shot[0], shot[1] < 10`.
- Recompute `pedersen(ships)` against the public hash.
- Expand ships to cells and compare against `shot_x + 10 * shot_y`; `hit` must
  match whether any cell matched.

### Takeaways

- Compact and readable, but the board hash has **no salt**. Valid placements
  number around 3×10¹⁰, which is small enough to brute-force against an
  unsalted hash. The Ragu version must include a random salt (blinding).
- Pairwise collision checks are quadratic. The bitmap-sum approach is linear.

## What we carry into the Ragu version

| Decision | Source | Why |
| --- | --- | --- |
| 100-cell bitmap with index `y * 10 + x` | Halo2 | Linear overlap check, cheap packing into one field element |
| Overlap check: every cell sum is boolean | Halo2 | One gate per cell |
| Salted commitment | Halo2 trapdoor | Without it, brute force reveals the board |
| Poseidon instead of Pedersen | new | Ragu ships an in-circuit Poseidon sponge (~288 gates per permutation); there is no fixed-base ECC chip to port |
| Placement legality by one-hot selection from all legal placements | new | Turns placement into a linear combination of precomputed masks; see [03-circuit-design.md](03-circuit-design.md) |
| Shot proves against the board PCD, not separately | new | Ragu's recursion lets the shot proof carry board validity with it |

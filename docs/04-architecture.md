# Architecture (proposal)

## Repository layout (target)

```
crates/
  battlezips/          # headers, steps, placement masks, native helpers, tests
  battlezips-wasm/     # wasm-bindgen API around one finalized Application
relay/                 # minimal WebSocket room relay (no game logic, no proofs)
web/                   # Vite + TypeScript UI; proving runs in a Web Worker
docs/
```

`battlezips` pins Ragu as a git dependency at a fixed commit. Both clients must
build a byte-identical `Application`: same steps, registration order, rank
(`R<13>`), header size (`4`), and registry tags.

## Proof flow

```mermaid
sequenceDiagram
    participant A as Alice (browser)
    participant R as Relay
    participant B as Bob (browser)
    Note over A: place fleet, pick salt
    A->>A: seed(BoardStep) → board_pcd_A (kept local, uncompressed)
    A->>A: compress(board_pcd_A)
    A->>R: commit { C_A, board_proof_A }
    R->>B: commit { C_A, board_proof_A }
    B->>B: verify_compressed, check header == C_A
    Note over B: same in the other direction
    B->>R: shot { x, y }
    R->>A: shot { x, y }
    A->>A: fuse(ShotStep, board_pcd_A, unit) → compress
    A->>R: answer { x, y, proof }
    R->>B: answer
    B->>B: verify_compressed; header == (C_A, 10y+x, hit)
    Note over B: tally hit; 17 verified hits = win
```

Rules for the client:

- **Only compressed proofs leave the browser.** Uncompressed PCDs carry the
  witness (see [02-ragu-primer.md](02-ragu-primer.md#zero-knowledge-status)).
- The board PCD (uncompressed) and the salt stay in the player's own worker
  for the whole game.
- A received proof is accepted only if it verifies **and** its header matches
  the expected `C` and shot index. Otherwise the turn is rejected.

## Messages (JSON over WebSocket)

| Type | Payload | Sender |
| --- | --- | --- |
| `join` | `{ room }` | client |
| `commit` | `{ commitment, proof }` | client, once |
| `shot` | `{ x, y }` | shooter |
| `answer` | `{ x, y, hit, proof }` | defender |
| `game_over` | `{ winner }` | either, informational |

Field elements are hex, and proofs are base64 of `CompressedProof::to_bytes()`.
The relay only forwards messages within a room of two. All validation happens
in the clients.

## WASM API (sketch)

```ts
init(): Promise<void>                       // build Application, load baked params + bootstrap proof
proveBoard(ships: Placement[]): { commitment, proof }   // also caches board PCD + salt in worker
proveShot(x: number, y: number): { hit, proof }
verifyBoard(proof, commitment): boolean
verifyShot(proof, commitment, x, y): { valid, hit }
```

Cost notes:

- `finalize` builds a bootstrap proof (one fuse) unless one is supplied. Build
  it once at compile time and load it with
  `ApplicationBuilder::with_bootstrap`.
- Each turn costs the defender one fuse plus one compress, and the shooter one
  `verify_compressed`. Measure these natively first, then in WASM.
- Run all proving in a dedicated Web Worker so the UI never blocks.

## Out of scope for the PoC

- Smart contracts and on-chain settlement.
- The phase-2 game-state chain (see [03-circuit-design.md](03-circuit-design.md#phase-2-later-game-state-chain)).
- Cheating resistance at the protocol level: timeouts, refusing to answer, and
  equivocating across two commitments. Clients are trusted to follow turn
  order.
- Privacy guarantees, until Ragu provides zero knowledge.

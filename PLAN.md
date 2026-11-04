# Plan

Each phase ends with something runnable and a short write-up of measured
numbers in `docs/`.

## Phase 0: Exploration ✅

- [x] Review BattleZips Halo2 and Noir circuits ([docs/01](docs/01-prior-art.md))
- [x] Read Ragu's book and API ([docs/02](docs/02-ragu-primer.md))
- [x] Draft circuit and architecture designs ([docs/03](docs/03-circuit-design.md), [docs/04](docs/04-architecture.md))

## Phase 1: Ragu hello-world + toolchain

- [ ] Pin Rust 1.97.1 (`rust-toolchain.toml`) and add `wasm32-unknown-unknown`
- [ ] Cargo workspace with `crates/battlezips` depending on Ragu at `3219e086` (git)
- [ ] Port the book's Merkle example as a smoke test (seed, fuse, compress, verify)
- [ ] Measure natively: seed, fuse, compress, verify times; compressed and uncompressed proof byte sizes
- [ ] Build the same smoke test for `wasm32-unknown-unknown` (no `multicore`); record bundle size and timings in a browser
- [ ] Confirm the multi-element `Header::Output` form and the unit right child (`bootstrap_proof().carry(())`)

**Exit:** a numbers table in `docs/05-benchmarks.md`, and a go/no-go on in-browser proving.

## Phase 2: Circuits

- [ ] Placement masks: generate the 760 placement masks (const or `build.rs`), with unit tests
- [ ] Native reference: `Board`, `commit(board, salt)`, `shot(board, i)`
- [ ] `BoardHeader`, `BoardStep`; measure real gate count with the metrics driver
- [ ] `ShotHeader`, `ShotStep`
- [ ] Negative tests: overlapping ships, off-board placement, wrong hit flag, wrong salt, mismatched commitment, two-hot shot selector
- [ ] CLI demo: two in-process players play a full game with compressed proofs passed as bytes

**Exit:** `cargo test` covers honest and dishonest paths; real gate counts recorded.

## Phase 3: WASM + web app

- [ ] `battlezips-wasm` with the API in [docs/04](docs/04-architecture.md#wasm-api-sketch)
- [ ] Precompute the bootstrap proof and ship it with the bundle
- [ ] `relay/`: two-player room WebSocket relay
- [ ] `web/`: fleet placement UI, opponent grid, proving worker, proof status/log panel
- [ ] Two-browser playthrough

**Exit:** two people can play a full game across machines.

## Phase 4: Beyond the PoC (not scheduled)

- Game-state PCD chain (hit tally, no repeat shots) for succinct win/loss proofs
- Settlement contract or state channel (revisit the Plonky2 ZK state channel idea)
- Switch to hiding proofs once Ragu supports zero knowledge
- Multithreaded WASM (`wasm-bindgen-rayon`, COOP/COEP)

## Open questions

1. **Zero knowledge.** Is a non-hiding demo acceptable for v1? Should the UI say
   "not private" explicitly? (Recommendation: yes, show a banner.)
2. **Browser feasibility.** If WASM fuse + compress is too slow (more than ~10 s
   per turn), fall back to a native prover sidecar, or an in-relay prover for the demo only?
3. **Ragu version.** Track `main` with a pinned rev and bump deliberately, or
   wait for a tagged release?
4. **Header binding.** Add a session id to `ShotHeader` to bind proofs to one
   game? (Cheap: one element, and `HEADER_SIZE = 4` has room.)
5. **Relay.** Rust (axum, to share types) or a ~50-line Node/Bun relay?

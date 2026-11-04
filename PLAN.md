# Plan

Each phase ends with something runnable and a short write-up of measured
numbers in `docs/`.

## Phase 0: Exploration ✅

- [x] Review BattleZips Halo2 and Noir circuits ([docs/01](docs/01-prior-art.md))
- [x] Read Ragu's book and API ([docs/02](docs/02-ragu-primer.md))
- [x] Draft circuit and architecture designs ([docs/03](docs/03-circuit-design.md), [docs/04](docs/04-architecture.md))

## Phase 1: Ragu hello-world + toolchain ✅

- [x] Pin Rust 1.97.1 (`rust-toolchain.toml`) with `wasm32-unknown-unknown`
- [x] Cargo workspace with `crates/battlezips` tracking Ragu `main` (git; `Cargo.lock` pins `3219e086`)
- [x] Smoke app: seed, fuse, compress, verify, plus byte round-trip and wrong-header rejection
- [x] Native timings and proof sizes
- [x] wasm32 build with no source changes; timings under Node/V8 (`crates/wasm-bench`)
- [x] Multi-element headers: tuple `Kind![Fp; (Element, Element, ..)]` is used upstream

**Result ([docs/05](docs/05-benchmarks.md)):** go. 7.3 KB compressed proofs, about 9.4 s per answered shot in single-threaded wasm, and 0.9 s to verify. Speedup work is tracked in Phase 3.

## Phase 2: Circuits

- [ ] Placement masks: generate the 760 placement masks (const or `build.rs`), with unit tests
- [ ] Native reference: `Board`, `commit(board, salt)`, `shot(board, i)`
- [ ] `BoardHeader`, `BoardStep`; measure real gate count with the metrics driver
- [ ] `ShotHeader`, `ShotStep` (confirm unit right child via `bootstrap_proof().clone().carry(())`)
- [ ] Negative tests: overlapping ships, off-board placement, wrong hit flag, wrong salt, mismatched commitment, two-hot shot selector
- [ ] CLI demo: two in-process players play a full game with compressed proofs passed as bytes

**Exit:** `cargo test` covers honest and dishonest paths; real gate counts recorded.

## Phase 3: WASM + web app

- [ ] `battlezips-wasm` with the API in [docs/04](docs/04-architecture.md#wasm-api-sketch)
- [ ] Precompute the bootstrap proof and ship it with the bundle
- [ ] `relay/`: two-player room WebSocket relay
- [ ] `web/`: fleet placement UI, opponent grid, proving worker, proof status/log panel
- [ ] Two-browser playthrough
- [ ] Browser threads via `wasm-bindgen-rayon` (COOP/COEP) + Ragu `multicore`
- [ ] Open an upstream issue: wasm32 field kernel (64-bit limbs × `u128` products are emulated; ~4.6× slower than native 1-thread)

**Exit:** two people can play a full game across machines.

## Phase 4: Beyond the PoC (not scheduled)

- Game-state PCD chain (hit tally, no repeat shots) for succinct win/loss proofs
- Settlement contract or state channel (revisit the Plonky2 ZK state channel idea)
- Switch to hiding proofs once Ragu supports zero knowledge
- Multithreaded WASM (`wasm-bindgen-rayon`, COOP/COEP)

## Open questions

1. **Zero knowledge.** Is a non-hiding demo acceptable for v1? Should the UI say
   "not private" explicitly? (Recommendation: yes, show a banner.)
2. ~~Browser feasibility~~: proving runs in the browser (~9.4 s per turn single-threaded). Make it faster rather than fall back.
3. ~~Ragu version~~: track `main`, pinned by `Cargo.lock`.
4. **Header binding.** Add a session id to `ShotHeader` to bind proofs to one
   game? (Cheap: one element, and `HEADER_SIZE = 4` has room.)
5. **Relay.** Rust (axum, to share types) or a ~50-line Node/Bun relay?

# Phase 1 benchmarks

The smoke application in `crates/battlezips/src/smoke.rs` has two steps:

- `Leaf`, a seed step that Poseidon-hashes its witness.
- `Merge`, a fuse step that Poseidon-hashes two `Value` headers.

It runs at `ProductionRank` (`R<13>`) with `HEADER_SIZE = 4`. The benchmark
seeds two leaves, merges them, compresses the root, and verifies it. Since step
size barely affects proving time (see [02-ragu-primer.md](02-ragu-primer.md)),
these numbers should be close to the real board and shot steps.

- Ragu: `main` @ `3219e086`
- Machine: Apple M4 Max, 16 cores
- Rust: 1.97.1

## Results

| Phase | Native, multicore | Native, 1 thread | WASM (Node 24 / V8), 1 thread |
| --- | ---: | ---: | ---: |
| `finalize` (registry + bootstrap fuse) | 831 ms | 2,501 ms | 11,431 ms |
| `seed` | 507 ms | 1,231 ms | 5,637 ms |
| `fuse` | 509 ms | 1,224 ms | 5,644 ms |
| `compress` | 336 ms | 773 ms | 3,747 ms |
| `verify_compressed` | 169 ms | 207 ms | 919 ms |

| Artifact | Size |
| --- | ---: |
| Uncompressed proof (`Proof::to_bytes`) | 24,120,832 B (23 MiB) |
| Compressed proof (`CompressedProof::to_bytes`) | **7,264 B** |
| WASM module (`fat` LTO, `simd128`, not wasm-opt'd) | 6.8 MiB |

The WASM column uses `CARGO_PROFILE_RELEASE_LTO=fat`, `CODEGEN_UNITS=1`, and
`+simd128`. Those flags gave about 3% over a plain release build (5,860 →
5,644 ms per fuse). Node runs on V8, so Chrome should be in the same range.

## What this means for a turn

| Who | Work | Native (MT) | WASM (1 thread) |
| --- | --- | ---: | ---: |
| Defender | fuse `ShotStep` + compress | ~0.85 s | **~9.4 s** |
| Shooter | decode + `verify_compressed` | ~0.17 s | ~0.9 s |
| Each player, once | board seed + compress | ~0.85 s | ~9.4 s |
| Each player, page load | `finalize` | ~0.8 s | ~11.4 s |

The game is playable in the browser as is, but slow: about 10 s per answered
shot. Proofs are small enough to send over a WebSocket without issue (7 KB).
**The uncompressed proof must never be sent.** It is 23 MiB and carries the witness.

## Why WASM is ~4.6× slower than native single-threaded

Udon's Pasta field arithmetic uses 64-bit limbs with `u128` intermediate
products. The only special-cased kernel is optional aarch64 inline asm
(`aarch64-asm` feature). `wasm32` has no 64×64→128 multiply, so every limb
product is emulated with several `i64.mul`s. A usual WASM penalty is 1.5–2×.

## Optimization levers, roughly in order of payoff

1. **Threads in the browser.** `wasm-bindgen-rayon` plus COOP/COEP headers, with
   Ragu's `multicore` feature on. Native MT vs 1-thread is 2.4× for fuse.
2. **A 32-bit-limb (or `i64`-friendly) field kernel for wasm32 in Udon.** This is
   the root cause of the 4.6× gap and is an upstream change. Worth an issue on
   `zakura` / `ragu`.
3. **Ship the bootstrap proof** (`ApplicationBuilder::with_bootstrap`) to skip
   one fuse at load time. It is 23 MiB uncompressed, so measure it against
   building it in a background worker while the player places ships.
4. Start the `ShotStep` fuse speculatively? Not possible, since the shot is
   unknown until it arrives. But the defender can prove while the UI animates.

## Reproduce

```sh
# native, multicore
cargo run --release --example smoke
# native, single thread
cargo run --release --example smoke --no-default-features
# wasm (raw exports, no wasm-bindgen needed)
CARGO_PROFILE_RELEASE_LTO=fat CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
  RUSTFLAGS="-C target-feature=+simd128" \
  cargo build --release --target wasm32-unknown-unknown -p wasm-bench
node crates/wasm-bench/bench.mjs
```

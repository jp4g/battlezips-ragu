# Ragu primer for BattleZips

Notes from reading `tachyon-zcash/ragu` at `3219e086` (book in `book/src`,
crates in `crates/`). This focuses on what affects a Battleship design.

## What Ragu is

Ragu is a **proof-carrying data** framework: a Halo-style recursive SNARK over
the Pasta cycle (Pallas/Vesta). It needs no trusted setup and is `no_std`
(it needs only an allocator). There is no "prove one circuit" API at the top
level. Everything is a **step** in a binary PCD tree.

| Concept | Type | Role |
| --- | --- | --- |
| Header | `ragu_pcd::header::Header` | The public state a proof certifies. Has a unique `SUFFIX`, a native `Data` type, and an in-circuit `Output` gadget, plus `encode`. |
| Step | `ragu_pcd::step::Step` | A transition. Declares `Left`, `Right`, and `Output` header types, a private `Witness`, and an `Aux` return. Its `witness()` function is the circuit. |
| Seed | `Application::seed` | Runs a step whose `Left = Right = ()`, a leaf. |
| Fuse | `Application::fuse` | Runs a step over two existing PCDs and outputs a new one. Both children's proofs are checked recursively. |
| Compress | `Application::compress` | Turns an uncompressed PCD into a much smaller, **terminal** proof (cannot be fused again). |
| Verify | `Application::verify_compressed` | The production verifier. `verify_uncompressed` is test-only (`test-utils`). |
| Wire format | `Proof::to_bytes` / `CompressedProof::to_bytes` | A versioned envelope with a fixed-length payload (optional `serde`). |

The application is built once:

```rust
let pasta = ragu_pcd::pasta::baked();             // feature "baked"
let app = ApplicationBuilder::<Pasta, R<13>, 4>::new()   // cycle, rank, header size
    .register(BoardStep { .. })?
    .register(ShotStep { .. })?
    .finalize(pasta)?;
```

Both players must build the **same** application (same steps, same order,
same rank and header size), or proofs will not verify across clients.
Registry tags need a fixed value for testing (`insecure-test-registry-tag` on
`ragu_circuits`) and should come from `RegistryTags::from_beacon` for anything
real.

## Arithmetization and budget

- Each **gate** has four wires `a, b, c, d` and enforces `a·b = c` and `c·d = 0`.
- There are `4n` **linear constraints** over all wires. Linear combinations
  (`Element::add`, `scale`, `multipack`) are virtual and add no gates.
- `R<13>` (the production rank) allows **2,048 gates and 8,192 constraints per
  step**, including fixed framework overhead.

Measured costs from the book (`guide/capacity.md`) and source:

| Operation | Cost |
| --- | --- |
| `Boolean::alloc` | 1 gate, 2 constraints |
| `Element::mul` | 1 gate |
| Poseidon permutation | 288 gates (4 squeezes per permutation) |
| Endoscalar op | 455 gates |
| Empty step | 2 gates |

Proving time is driven by **node count, not step size**. A 2-gate step and a
2,018-gate step both took ~200 ms per fuse and ~350 ms per seed (release build,
multicore, one developer machine). So the right design fills each step and
keeps the number of steps low. Splitting a step is only worth it when it
exceeds the rank.

## Useful gadgets (`ragu_primitives`)

- `Element`: `alloc`, `constant`, `add`, `sub`, `mul`, `scale`, `sum`,
  `is_zero`, `is_equal`, `enforce_zero`, `invert`, `fold`.
- `Boolean`: `alloc`, `not` (free), `and`, `conditional_select`,
  `conditional_enforce_equal`, and `multipack(bits)`, which packs up to
  `CAPACITY` (254 for Pallas) bits into one `Element` with no gates.
- `poseidon::Sponge`: `new(dr, params)`, `absorb`, `squeeze`.
- `ragu_gadgets` is currently empty, so there is no ready-made range check or
  bit decomposition. We build what we need from `Boolean`.

## Zero-knowledge status

**This is the most important finding.** Upstream is clear that Ragu is not
zero-knowledge yet:

- `Proof` (uncompressed) "contains trace polynomials", which is the witness. An
  uncompressed shot proof sent to an opponent **directly reveals the board**.
- `compress/mod.rs`: "Like an uncompressed proof, a compressed proof is not
  hiding: the openings it carries are evaluations of the witness polynomials."
- `Application::rerandomize`: unlinkability "is not achieved at all". The output
  embeds the input's commitments and stage polynomials in the clear. Witness
  hiding needs a fold-level randomizer, which is future work. See also
  [ragu#242](https://github.com/tachyon-zcash/ragu/issues/242).
- The book's PCD chapter: "The current construction does not yet establish zero
  knowledge or unlinkability."

What this means for the demo:

1. **Never send uncompressed proofs to the opponent.** Only compressed proofs go
   over the wire.
2. Compressed proofs still leak a bounded number of linear evaluations of the
   trace. Recovering a board from them is not trivial, especially with a
   field-element salt and Poseidon internals in the witness, but it is **not
   secure** and must not be described that way.
3. The PoC is about correctness, ergonomics, and performance. Privacy becomes
   real when upstream lands hiding. We should track that and re-check before
   making any privacy claim.

## Other risks and unknowns

| Item | Notes |
| --- | --- |
| Rust 1.97.1 | Pinned by `rust-toolchain.toml`. Edition 2024. |
| WASM | ✅ Builds for `wasm32-unknown-unknown` unchanged (no `multicore`). ~4.6× slower than native single-threaded because field multiplication emulates `u128` products. See [05-benchmarks.md](05-benchmarks.md). |
| Browser proving time | ✅ Measured: WASM fuse 5.6 s, compress 3.7 s, verify 0.9 s (M4 Max, V8). |
| Proof sizes | ✅ Uncompressed 23 MiB, compressed 7,264 B. |
| Baked generators | The `baked` feature embeds 2¹³ generators per curve in the binary. This affects WASM bundle size. |
| API churn | 3,000+ commits and active (`pushed_at` today). Pin a commit and use a git dependency. Crates.io `0.1` (used in the book) may lag `main`. |
| Multi-element headers | ✅ Upstream uses tuples: `Kind![Fp; (Element<'_, _>, Element<'_, _>)]`. |
| Unit right child for fuse | A step with `Left = Board`, `Right = ()` needs a `Pcd<()>`. `app.bootstrap_proof().clone().carry(())` should work. Confirm it is accepted as an ordinary child. |

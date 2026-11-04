//! Times seed / fuse / compress / verify for the smoke application and prints
//! proof sizes. Run with `cargo run --release --example smoke`.

use std::time::Instant;

use battlezips::smoke::{Leaf, Merge, Value};
use ragu_circuits::polynomials::ProductionRank;
use ragu_core::pasta::{Fp, Pasta};
use ragu_pcd::{ApplicationBuilder, CompressedProof, Proof};
use rand::{SeedableRng, rngs::StdRng};

fn time<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let out = f();
    println!("{label:<24} {:>8.1} ms", start.elapsed().as_secs_f64() * 1e3);
    out
}

fn main() -> ragu_core::Result<()> {
    let mut rng = StdRng::seed_from_u64(1);
    let params = time("load baked params", ragu_pcd::pasta::baked);
    let app = time("finalize (bootstrap)", || {
        ApplicationBuilder::<Pasta, ProductionRank, 4>::new()
            .register(Leaf)?
            .register(Merge)?
            .finalize(params)
    })?;

    let (a, ()) = time("seed", || app.seed(&mut rng, Leaf, Fp::from(100u64)))?;
    let (b, ()) = time("seed", || app.seed(&mut rng, Leaf, Fp::from(200u64)))?;
    let (root, ()) = time("fuse", || app.fuse(&mut rng, Merge, (), a, b))?;
    let compressed = time("compress", || app.compress(&root, &mut rng))?;
    let ok = time("verify_compressed", || app.verify_compressed(&compressed))?;
    assert!(ok, "compressed proof must verify");

    let uncompressed_bytes = root.proof().to_bytes().expect("encode proof");
    let compressed_bytes = compressed.proof().to_bytes().expect("encode compressed");
    println!("uncompressed proof       {:>8} bytes", uncompressed_bytes.len());
    println!("compressed proof         {:>8} bytes", compressed_bytes.len());

    // Round-trip as a remote verifier would: decode bytes, attach expected header data.
    let decoded = CompressedProof::<Pasta>::from_bytes(&compressed_bytes).expect("decode");
    let expected = *root.data();
    let ok = time("decode + verify", || {
        app.verify_compressed(&decoded.clone().carry::<Value>(expected))
    })?;
    assert!(ok);
    let wrong = app.verify_compressed(&decoded.carry::<Value>(expected + Fp::from(1u64)))?;
    assert!(!wrong, "wrong header data must not verify");
    let _ = Proof::<Pasta, ProductionRank>::from_bytes(&uncompressed_bytes).expect("decode");
    println!("ok");
    Ok(())
}

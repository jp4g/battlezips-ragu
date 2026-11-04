//! Raw `extern "C"` exports so a JS host can time each proving phase of the
//! smoke application without wasm-bindgen. See `bench.mjs`.

use std::cell::RefCell;

use battlezips::smoke::{Leaf, Merge, Value};
use ragu_circuits::polynomials::ProductionRank;
use ragu_core::pasta::{Fp, Pasta};
use ragu_pcd::{Application, ApplicationBuilder, CompressedPcd, Pcd};
use rand::{SeedableRng, rngs::StdRng};

type App = Application<'static, Pasta, ProductionRank, 4>;

#[derive(Default)]
struct State {
    app: Option<App>,
    rng: Option<StdRng>,
    leaves: Vec<Pcd<Pasta, ProductionRank, Value>>,
    root: Option<Pcd<Pasta, ProductionRank, Value>>,
    compressed: Option<CompressedPcd<Pasta, Value>>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::default();
}

fn with<T>(f: impl FnOnce(&mut State) -> T) -> T {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

#[unsafe(no_mangle)]
pub extern "C" fn finalize() {
    with(|s| {
        s.rng = Some(StdRng::seed_from_u64(1));
        s.app = Some(
            ApplicationBuilder::<Pasta, ProductionRank, 4>::new()
                .register(Leaf)
                .unwrap()
                .register(Merge)
                .unwrap()
                .finalize(ragu_pcd::pasta::baked())
                .unwrap(),
        );
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn seed(x: u64) {
    with(|s| {
        let (pcd, ()) = s
            .app
            .as_ref()
            .unwrap()
            .seed(s.rng.as_mut().unwrap(), Leaf, Fp::from(x))
            .unwrap();
        s.leaves.push(pcd);
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn fuse() {
    with(|s| {
        let b = s.leaves.pop().unwrap();
        let a = s.leaves.pop().unwrap();
        let (pcd, ()) = s
            .app
            .as_ref()
            .unwrap()
            .fuse(s.rng.as_mut().unwrap(), Merge, (), a, b)
            .unwrap();
        s.root = Some(pcd);
    })
}

/// Returns the compressed proof's encoded byte length.
#[unsafe(no_mangle)]
pub extern "C" fn compress() -> u32 {
    with(|s| {
        let c = s
            .app
            .as_ref()
            .unwrap()
            .compress(s.root.as_ref().unwrap(), s.rng.as_mut().unwrap())
            .unwrap();
        let len = c.proof().to_bytes().unwrap().len() as u32;
        s.compressed = Some(c);
        len
    })
}

/// Returns 1 if the compressed proof verifies.
#[unsafe(no_mangle)]
pub extern "C" fn verify() -> u32 {
    with(|s| {
        s.app
            .as_ref()
            .unwrap()
            .verify_compressed(s.compressed.as_ref().unwrap())
            .unwrap() as u32
    })
}

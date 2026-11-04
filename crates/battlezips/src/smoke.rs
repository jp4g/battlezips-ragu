//! Minimal Ragu application used to check toolchain, timings and proof sizes:
//! a seed step hashing a witness into a [`Value`], and a merge step hashing two
//! [`Value`]s together.

use ragu_core::{
    Cycle, Result,
    drivers::{Driver, DriverValue},
    gadgets::{Bound, Kind},
    maybe::Maybe,
    pasta::{Fp, Pasta},
};
use ragu_pcd::{
    header::{Header, Suffix},
    step::{Encoded, Index, Step},
};
use ragu_primitives::{
    Element,
    allocator::{Allocator, Standard},
    poseidon::Sponge,
};

/// A header carrying one field element.
pub struct Value;

impl Header<Fp> for Value {
    const SUFFIX: Suffix = Suffix::new(0);
    type Data = Fp;
    type Output = Kind![Fp; Element<'_, _>];

    fn encode<'dr, D: Driver<'dr, F = Fp>, A: Allocator<'dr, D>>(
        dr: &mut D,
        allocator: &mut A,
        witness: DriverValue<D, Self::Data>,
    ) -> Result<Bound<'dr, D, Self::Output>> {
        Element::alloc(dr, allocator, witness)
    }
}

/// Seed: `Value(H(witness))`.
pub struct Leaf;

impl Step<Pasta> for Leaf {
    const INDEX: Index = Index::new(0);
    type Witness<'source> = Fp;
    type Aux<'source> = ();
    type Left = ();
    type Right = ();
    type Output = Value;

    fn witness<'dr, 'source: 'dr, D: Driver<'dr, F = Fp>, const HS: usize>(
        &self,
        dr: &mut D,
        witness: DriverValue<D, Fp>,
        left: DriverValue<D, ()>,
        right: DriverValue<D, ()>,
    ) -> Result<(
        (
            Encoded<'dr, D, (), HS>,
            Encoded<'dr, D, (), HS>,
            Encoded<'dr, D, Value, HS>,
        ),
        DriverValue<D, Fp>,
        DriverValue<D, ()>,
    )>
    where
        Self: 'dr,
    {
        let allocator = &mut Standard::new();
        let left = Encoded::new(dr, allocator, left)?;
        let right = Encoded::new(dr, allocator, right)?;
        let x = Element::alloc(dr, allocator, witness)?;
        let mut sponge = Sponge::new(dr, Pasta::circuit_poseidon(ragu_pcd::pasta::baked()));
        sponge.absorb(dr, &x)?;
        let out = sponge.squeeze(dr)?;
        let data = out.value().map(|v| *v);
        Ok(((left, right, Encoded::from_gadget(out)), data, D::unit()))
    }
}

/// Fuse: `Value(H(left, right))`.
pub struct Merge;

impl Step<Pasta> for Merge {
    const INDEX: Index = Index::new(1);
    type Witness<'source> = ();
    type Aux<'source> = ();
    type Left = Value;
    type Right = Value;
    type Output = Value;

    fn witness<'dr, 'source: 'dr, D: Driver<'dr, F = Fp>, const HS: usize>(
        &self,
        dr: &mut D,
        _: DriverValue<D, ()>,
        left: DriverValue<D, Fp>,
        right: DriverValue<D, Fp>,
    ) -> Result<(
        (
            Encoded<'dr, D, Value, HS>,
            Encoded<'dr, D, Value, HS>,
            Encoded<'dr, D, Value, HS>,
        ),
        DriverValue<D, Fp>,
        DriverValue<D, ()>,
    )>
    where
        Self: 'dr,
    {
        let allocator = &mut Standard::new();
        let left = Encoded::new(dr, allocator, left)?;
        let right = Encoded::new(dr, allocator, right)?;
        let mut sponge = Sponge::new(dr, Pasta::circuit_poseidon(ragu_pcd::pasta::baked()));
        sponge.absorb(dr, left.as_gadget())?;
        sponge.absorb(dr, right.as_gadget())?;
        let out = sponge.squeeze(dr)?;
        let data = out.value().map(|v| *v);
        Ok(((left, right, Encoded::from_gadget(out)), data, D::unit()))
    }
}

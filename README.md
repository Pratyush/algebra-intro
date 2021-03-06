# Introduction to `arkworks` algebra APIs

The [`arkworks` ecosystem](https://arkworks.rs) is a set of state-of-the-art Rust libraries that collectively provide tools to program zkSNARKs.
zkHack puzzles will be using `arkworks` libraries for elliptic curve and finite field arithmetic. This document is a helpful cheat-sheet to get started with using these libraries.

## Finite field arithmetic

There are three important traits when working with finite fields: [`Field`],
[`SquareRootField`], and [`PrimeField`]. Let's explore these via examples.

### [`Field`]

The [`Field`] trait provides a generic interface for any finite field.
Types implementing [`Field`] support common field operations
such as addition, subtraction, multiplication, and inverses.

```rust
use ark_ff::Field;
// We'll use a field associated with the BLS12-381 pairing-friendly
// group for this example.
use ark_bls12_381::Fq2 as F;
// `ark-std` is a utility crate that enables `arkworks` libraries
// to easily support `std` and `no_std` workloads, and also re-exports
// useful crates that should be common across the entire ecosystem, such as `rand`.
use ark_std::{One, UniformRand};

let mut rng = ark_std::rand::thread_rng();
// Let's sample uniformly random field elements:
let a = F::rand(&mut rng);
let b = F::rand(&mut rng);

// We can add...
let c = a + b;
// ... subtract ...
let d = a - b;
// ... double elements ...
assert_eq!(c + d, a.double());

// ... multiply ...
let e = c * d;
// ... square elements ...
assert_eq!(e, a.square() - b.square());

// ... and compute inverses ...
assert_eq!(a.inverse().unwrap() * a, F::one()); // have to to unwrap, as `a` could be zero.
```

### [`SquareRootField`]

In some cases, it is important to take square roots of field elements
(e.g.: for point compression of elliptic curve elements.)
To support this, users can implement the [`SquareRootField`] trait for their field type. This
provides access to the following methods:

```rust
use ark_ff::{Field, SquareRootField};
// As before, we'll use a field associated with the BLS12-381 pairing-friendly
// group for this example.
use ark_bls12_381::Fq2 as F;
use ark_std::{One, UniformRand};

let mut rng = ark_std::rand::thread_rng();
// Let's try to sample a random square via rejection sampling:
let mut a = F::rand(&mut rng);
while a.legendre().is_qnr() { // A square is also called a *quadratic residue*
    a = F::rand(&mut rng);
}

// Since `a` is a square, we can compute its square root:
let b = a.sqrt().unwrap();
assert_eq!(b.square(), a);

// Let's sample a random *non-square*
let mut a = F::rand(&mut rng);
while a.legendre().is_qr() {
    a = F::rand(&mut rng);
}
// The square root should not exist:
assert_eq!(a.sqrt(), None);
```

### [`PrimeField`]

If the field is of prime order, then users can choose
to implement the [`PrimeField`] trait for it. This provides access to the following
additional APIs:

```rust
use ark_ff::{Field, PrimeField, FpParameters, BigInteger};
// Now we'll use the prime field underlying the BLS12-381 G1 curve.
use ark_bls12_381::Fq as F;
use ark_std::{One, Zero, UniformRand};

let mut rng = ark_std::rand::thread_rng();
let a = F::rand(&mut rng);
// We can access the prime modulus associated with `F`:
let modulus = <F as PrimeField>::Params::MODULUS;
assert_eq!(a.pow(&modulus), a);

// We can convert field elements to integers in the range [0, MODULUS - 1]:
let one: num_bigint::BigUint = F::one().into();
assert_eq!(one, num_bigint::BigUint::one());

// We can construct field elements from an arbitrary sequence of bytes:
let n = F::from_le_bytes_mod_order(&modulus.to_bytes_le());
assert_eq!(n, F::zero());
```

## Elliptic curve arithmetic

There are two traits that are important when working with elliptic curves
over finite fields: [`ProjectiveCurve`], and [`AffineCurve`]. Both traits
represent the same curve, but provide different underlying representations.
In particular, a [`ProjectiveCurve`] representation of a curve point is generally
more efficient for arithmetic, but does not provide a unique representative
for a curve point. An [`AffineCurve`] representation, on the other hand, *is* unique,
but is slower for most arithmetic operations. Let's explore how and when to use
these:

```rust
use ark_ec::{ProjectiveCurve, AffineCurve};
use ark_ff::{PrimeField, Field};
// We'll use the BLS12-381 G1 curve for this example.
use ark_bls12_381::{G1Projective as G, G1Affine as GAffine, Fr as ScalarField};
use ark_std::{Zero, UniformRand};

let mut rng = ark_std::rand::thread_rng();
// Let's sample uniformly random field elements:
let a = G::rand(&mut rng);
let b = G::rand(&mut rng);

// We can add...
let c = a + b;
// ... subtract ...
let d = a - b;
// ... and double elements.
assert_eq!(c + d, a.double());
// We can also negate elements...
let e = -a;
assert_eq!(e + a, G::zero());

// ...and multiply group elements by elements of the corresponding scalar field
let scalar = ScalarField::rand(&mut rng);
let e = c.mul(&scalar.into_repr()); // into_repr() converts the scalar into a `BigInteger`.
let f = e.mul(&scalar.inverse().unwrap().into_repr());
assert_eq!(f, c);

// Finally, we can also convert curve points in projective coordinates to affine coordinates.
let c_aff = c.into_affine();
// Most group operations are slower in affine coordinates, but adding an affine point
// to a projective one is slightly more efficient.
let d = c.add_mixed(&c_aff);
assert_eq!(d, c.double());

// This efficiency also translates into more efficient scalar multiplication routines.
let e_from_aff = c_aff.mul(scalar.into_repr());
assert_eq!(e, e_from_aff);

// Finally, while not recommended, users can directly construct group elements
// from the x and y coordinates. This is useful when implementing algorithms
// like hash-to-curve.
let e_affine = e.into_affine();
let e_x = e_affine.x;
let e_y = e_affine.y;
let is_at_infinity = e_affine.is_zero();
let new_e = GAffine::new(e_x, e_y, is_at_infinity);
assert_eq!(e_affine, new_e);
// Users should check that the new point is on the curve and is in the prime-order group:
assert!(new_e.is_on_curve());
assert!(new_e.is_in_correct_subgroup_assuming_on_curve());
```

## Pairings

[`PairingEngine`] is the primary trait for working with pairings. It contains
associated types and methods that are relevant to pairing operations:

```rust
use ark_ec::{ProjectiveCurve, AffineCurve, PairingEngine};
use ark_ff::{PrimeField, Field};
// We'll use the BLS12-381 pairing-friendly group for this example.
use ark_bls12_381::{Bls12_381, G1Projective as G1, G2Projective as G2, G1Affine, G2Affine, Fr as ScalarField};
use ark_std::{Zero, UniformRand};

let mut rng = ark_std::rand::thread_rng();
// Let's sample uniformly random field elements:
let a: G1Affine = G1::rand(&mut rng).into();
let b: G2Affine = G2::rand(&mut rng).into();
// We can compute the pairing of `a` and `b`:
let c = Bls12_381::pairing(a, b);

// We can also compute the pairing partwise:
// First, we compute the Miller loop:
let c_ml = Bls12_381::miller_loop(&[(a.into(), b.into())]);
let c_fe = Bls12_381::final_exponentiation(&c_ml).unwrap();
assert_eq!(c, c_fe);
```

## Serialization

Most types in the `arkworks` ecosystem implement the [`CanonicalSerialize`]
and [`CanonicalDeserialize`] traits. These traits enable converting these types
to canonical byte representations that are suitable for disk storage and network
communication. They also enable support for point compression.

```rust
use ark_ec::AffineCurve;
// We'll use the BLS12-381 pairing-friendly group for this example.
use ark_bls12_381::{G1Projective as G1, G2Projective as G2, G1Affine, G2Affine};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
use ark_std::UniformRand;

let mut rng = ark_std::rand::thread_rng();
// Let's sample uniformly random field elements:
let a: G1Affine = G1::rand(&mut rng).into();
let b: G2Affine = G2::rand(&mut rng).into();

// We can serialize with compression...
let mut compressed_bytes = Vec::new();
a.serialize(&mut compressed_bytes).unwrap();
// ...and without:
let mut uncompressed_bytes = Vec::new();
a.serialize_uncompressed(&mut uncompressed_bytes).unwrap();

// We can reconstruct our points from the compressed serialization...
let a_compressed = G1Affine::deserialize(&*compressed_bytes).unwrap();

// ... and from the uncompressed one:
let a_uncompressed = G1Affine::deserialize_uncompressed(&*uncompressed_bytes).unwrap();

assert_eq!(a_compressed, a);
assert_eq!(a_uncompressed, a);

// If we trust the origin of the serialization
// (eg: if the serialization was stored on authenticated storage),
// then we can skip some validation checks:
let a_unchecked = G1Affine::deserialize_unchecked(&*uncompressed_bytes).unwrap();
assert_eq!(a_unchecked, a);
```

[`Field`]: https://docs.rs/ark-ff/0.3.0/ark_ff/fields/trait.Field.html
[`PrimeField`]: https://docs.rs/ark-ff/0.3.0/ark_ff/fields/trait.PrimeField.html
[`SquareRootField`]: https://docs.rs/ark-ff/0.3.0/ark_ff/fields/trait.SquareRootField.html
[`AffineCurve`]: https://docs.rs/ark-ec/0.3.0/ark_ec/trait.AffineCurve.html
[`ProjectiveCurve`]: https://docs.rs/ark-ec/0.3.0/ark_ec/trait.ProjectiveCurve.html
[`PairingEngine`]: https://docs.rs/ark-ec/0.3.0/ark_ec/trait.PairingEngine.html
[`CanonicalSerialize`]: https://docs.rs/ark-serialize/0.3.0/ark_serialize/trait.CanonicalSerialize.html
[`CanonicalDeserialize`]: https://docs.rs/ark-serialize/0.3.0/ark_serialize/trait.CanonicalDeserialize.html

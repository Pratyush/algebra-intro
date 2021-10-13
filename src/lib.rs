#![allow(unused)]

//! # Introduction to `arkworks`
//! 
//! The [`arkworks` ecosystem](https://arkworks.rs) is a set of state-of-the-art Rust libraries that collectively provide tools to program zkSNARKs. 
//! zkHack puzzles will be using `arkworks` libraries for elliptic curve and finite field arithmetic. This document is a helpful cheat-sheet to get started with using these libraries.
//! 
//! ## Finite field arithmetic
//! There are three important traits when working with finite fields: [`Field`], 
//! [`SquareRootField`], and [`PrimeField`]. Let's explore these via examples.
//! 
//! ### [`Field`]
//! 
//! The [`Field`] trait provides a generic interface for any finite field. 
//! Types implementing [`Field`] support common field operations 
//! such as addition, subtraction, multiplication, and inverses.
//! 
//! ```rust
//! use ark_ff::Field;
//! // We'll use a field associated with the BLS12-381 pairing-friendly 
//! // group for this example.
//! use ark_bls12_381::Fq2 as F;
//! // `ark-std` is a utility crate that enables `arkworks` libraries
//! // to easily support `std` and `no_std` workloads, and also re-exports
//! // useful crates that should be common across the entire ecosystem, such as `rand`.
//! use ark_std::{One, UniformRand};
//! 
//! let mut rng = ark_std::rand::thread_rng();
//! // Let's sample uniformly random field elements:
//! let a = F::rand(&mut rng);
//! let b = F::rand(&mut rng);
//! 
//! // We can add...
//! let c = a + b;
//! // ... subtract ...
//! let d = a - b;
//! // ... double elements ...
//! assert_eq!(c + d, a.double());
//! 
//! // ... multiply ...
//! let e = c * d;
//! // ... square elements ...
//! assert_eq!(e, a.square() - b.square());
//! 
//! // ... and compute inverses ...
//! assert_eq!(a.inverse().unwrap() * a, F::one()); // have to to unwrap, as `a` could be zero.
//! ```
//! 
//! ### [`SquareRootField`]
//! 
//! In some cases, it is important to take square roots of field elements 
//! (eg: for point compression of elliptic curve elements.)
//! To support this, users can implement the [`SquareRootField`] trait for their field type. This
//! provides access to the following methods:
//! 
//! ```rust
//! use ark_ff::{Field, SquareRootField};
//! // As before, we'll use a field associated with the BLS12-381 pairing-friendly 
//! // group for this example.
//! use ark_bls12_381::Fq2 as F;
//! use ark_std::{One, UniformRand};
//! 
//! let mut rng = ark_std::rand::thread_rng();
//! // Let's try to sample a random square via rejection sampling:
//! let mut a = F::rand(&mut rng);
//! while a.legendre().is_qnr() { // A square is also called a *quadratic residue*
//!     a = F::rand(&mut rng);
//! }
//! 
//! // Since `a` is a square, we can compute its square root:
//! let b = a.sqrt().unwrap();
//! assert_eq!(b.square(), a);
//! 
//! // Let's sample a random *non-square*
//! let mut a = F::rand(&mut rng);
//! while a.legendre().is_qr() {
//!     a = F::rand(&mut rng);
//! }
//! // The square root should not exist:
//! assert_eq!(a.sqrt(), None);
//! ```
//! ### [`PrimeField`]
//! 
//! If the field is of prime order, then users can choose 
//! to implement the [`PrimeField`] trait for it. This provides access to the following 
//! additional APIs:
//! ```rust
//! use ark_ff::{Field, PrimeField, FpParameters, BigInteger};
//! // Now we'll use the prime field underlying the BLS12-381 G1 curve.
//! use ark_bls12_381::Fq as F;
//! use ark_std::{One, Zero, UniformRand};
//! 
//! let mut rng = ark_std::rand::thread_rng();
//! let a = F::rand(&mut rng);
//! // We can access the prime modulus associated with `F`:
//! let modulus = <F as PrimeField>::Params::MODULUS;
//! assert_eq!(a.pow(&modulus), a);
//! 
//! // We can convert field elements to integers in the range [0, MODULUS - 1]:
//! let one: num_bigint::BigUint = F::one().into();
//! assert_eq!(one, num_bigint::BigUint::one());
//! 
//! // We can construct field elements from an arbitrary sequence of bytes:
//! let n = F::from_le_bytes_mod_order(&modulus.to_bytes_le());
//! assert_eq!(n, F::zero());
//! ```

pub use ark_ff::{Field, SquareRootField, PrimeField};
// This file is part of midnight-trusted-setup.
// Copyright (C) 2025 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This binary verifies consistency between a powers-of-tau SRS and an extended
//! SRS file including both the coefficients and Lagrange representations.
//!
//! Concretely, it checks that:
//! 1. The G1 points of the powers-of-tau file coincide with the extended SRS's
//!    coefficient representation.
//! 2. The G2 points match between both files.
//! 3. The Lagrange basis in the extended SRS is correctly derived from the
//!    coefficient basis.
//!
//! Computing the Lagrange form of the SRS can be computationally intensive,
//! since it requires applying a long FFT "in the exponent".
//!
//! However, verifying consistency between both representations can be performed
//! significantly faster by sampling a random polynomial and committing to it in
//! both coefficient and Lagrange forms, ensuring both representations produce
//! identical commitments. This check would fail with overwhelming probability
//! if the representations were not consistent.
//!
//! Technically, verifiers only need consistency between the G2 points,
//! which can be checked by simply comparing the last 2 * 192 = 384 bytes of
//! both files. This can be done by e.g.
//!
//! ```bash
//! cmp -s <(tail -c 384 <PATH-TO-POWERS-OF-TAU>) \
//!        <(tail -c 384 <PATH-TO-EXTENDED-SRS>) \
//! && echo "ok" || echo "inconsistent"
//! ```
//!
//! However, provers also require the G1 points to be consistent. This binary
//! provides tools for verifying consistency between both the G1 and G2 points.

use std::{io::Read, path::Path};

use blstrs::{G1Affine, G2Affine};
use clap::Parser;
use ff::{Field, PrimeField};
use halo2curves::{fft::best_fft, msm::msm_best};
use rand_core::OsRng;
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSlice,
};
use srs::{
    ceremony::{G1_SIZE, G2_SIZE},
    utils::{compare_bytes, initialize_progress_bar, open_file, read_g1_point, read_g2_point},
};

type F = blstrs::Scalar;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the powers-of-tau ceremony file.
    powers_of_tau_path: String,

    /// Path to the extended SRS file (in both coefficient and Lagrange form).
    extended_srs_path: String,
}

/// Extended SRS containing both coefficient and Lagrange representations.
///
/// This structure holds KZG parameters in two bases:
/// - Coefficient form: `g1s_coeff := [1, τ, τ², ..., τⁿ⁻¹]₁`.
/// - Lagrange form: `g1s_lagrange := [L₀(τ), L₁(τ), ..., Lₙ₋₁(τ)]₁`.
///
/// where `Lᵢ` are the Lagrange basis polynomials over the n-th roots of unity.
///
/// It also holds `g2s := [1, τ]₂`, and `k := log₂(n)`.
struct ExtendedSRS {
    /// G1 points in coefficient (monomial) basis.
    g1s_coeff: Vec<G1Affine>,

    /// G1 points in Lagrange basis.
    g1s_lagrange: Vec<G1Affine>,

    /// G2 points: [1, τ]₂.
    _g2s: [G2Affine; 2],

    /// Log in base 2 of the SRS size.
    k: u32,
}

impl ExtendedSRS {
    fn read_from_file(path: &Path) -> Self {
        let mut file = open_file(path);
        let mut bytes = Vec::<u8>::new();
        file.read_to_end(&mut bytes).expect("Cannot read to end");

        let k = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        let n = 1 << k;

        assert_eq!(bytes.len(), 4 + 2 * n * G1_SIZE + 2 * G2_SIZE);

        let pb = initialize_progress_bar(2 * n, Some("Reading Lagrange SRS".into()));

        let mut offset = 4;

        let g1s_coeff: Vec<G1Affine> = bytes[offset..(offset + G1_SIZE * n)]
            .par_chunks(G1_SIZE)
            .inspect(|_| pb.inc(1))
            .map(read_g1_point)
            .collect::<Vec<_>>();
        offset += G1_SIZE * n;

        let g1s_lagrange: Vec<G1Affine> = bytes[offset..(offset + G1_SIZE * n)]
            .par_chunks(G1_SIZE)
            .inspect(|_| pb.inc(1))
            .map(read_g1_point)
            .collect::<Vec<_>>();
        offset += G1_SIZE * n;

        pb.finish_and_clear();

        let mut _g2s = [G2Affine::default(); 2];
        _g2s[0] = read_g2_point(&bytes[offset..(offset + G2_SIZE)]);
        _g2s[1] = read_g2_point(&bytes[(offset + G2_SIZE)..(offset + 2 * G2_SIZE)]);

        Self {
            g1s_coeff,
            g1s_lagrange,
            _g2s,
            k,
        }
    }

    /// Verifies that the Lagrange basis is consistent with the coefficient
    /// basis.
    ///
    /// This method samples a random polynomial and commits to it using both
    /// representations. If the commitments differ, the Lagrange basis was
    /// incorrectly derived. This probabilistic check would fail with
    /// overwhelming probability if the representations were inconsistent.
    fn check_consistency(&self) {
        let n = self.g1s_coeff.len();

        // Sample a uniformly random polynomial of degree < n.
        let mut random_poly: Vec<F> = (0..n).into_par_iter().map(|_| F::random(OsRng)).collect();

        // Commit to the polynomial in coefficients form.
        let com_coeff = msm_best::<G1Affine>(&random_poly, &self.g1s_coeff);

        // Commit to the polynomial in Lagrange form.
        let omega = F::ROOT_OF_UNITY.pow([1u64 << (F::S - self.k)]);
        best_fft(&mut random_poly, omega, self.k);
        let com_lagrange = msm_best::<G1Affine>(&random_poly, &self.g1s_lagrange);

        assert_eq!(
            com_coeff, com_lagrange,
            "The coefficients and Lagrange representations are inconsistent",
        );
    }
}

fn main() {
    let args = Args::parse();

    let path1 = Path::new(&args.powers_of_tau_path);
    let path2 = Path::new(&args.extended_srs_path);

    let srs = ExtendedSRS::read_from_file(path2);
    let n = srs.g1s_coeff.len();

    // 1. The G1 points of the powers-of-tau file coincide with the extended SRS's
    //    coefficient representation.
    assert!(
        compare_bytes(path1, path2, 0, 4, n * G1_SIZE),
        "G1 points mismatch between powers-of-tau and the extended SRS"
    );

    // 2. The G2 points match between both files.
    assert!(
        compare_bytes(
            path1,
            path2,
            -2 * G2_SIZE as i64,
            -2 * G2_SIZE as i64,
            2 * G2_SIZE
        ),
        "G2 points mismatch between powers-of-tau and the extended SRS"
    );

    // 3. The Lagrange basis in the extended SRS is correctly derived from the
    //    coefficient basis.
    srs.check_consistency();

    println!("All checks passed!")
}

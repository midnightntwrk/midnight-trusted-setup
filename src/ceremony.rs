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

use std::{
    io::{Read, Write},
    path::Path,
};

use blstrs::{pairing, G1Affine, G2Affine, Scalar};
use halo2curves::{
    ff::Field,
    group::{prime::PrimeCurveAffine, Curve},
    msm::msm_best,
    serde::SerdeObject,
};
use rand_core::OsRng;
use rayon::prelude::*;

use crate::{
    schnorr::UpdateProof,
    utils::{
        create_file, initialize_progress_bar, open_file, powers, read_g1_point, read_g2_point,
    },
};

// Size of (uncompressed) G1 and G2 points
// See: https://github.com/filecoin-project/powersoftau/blob/ab8f85c28f04af5a99cfcc93a3b1f74c06f94105/src/bls12_381/mod.rs#L52C1-L53C46
pub const G1_SIZE: usize = 96;
pub const G2_SIZE: usize = 192;
pub const SCALAR_SIZE: usize = 32;

#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub struct SRS {
    // Store [1]_1, [tau]_1, ..., [tau^N-1]
    pub g1s: Vec<G1Affine>,
    // Store [1]_2, [tau]_2
    pub g2s: [G2Affine; 2],
}

// Necessary functionality for Ceremony
impl SRS {
    /// Verifies the SRS structure. Panics if the structure is not correct
    pub fn verify_structure(&self) {
        assert!(
            self.g1s.par_iter().all(|&p| p != G1Affine::identity()),
            "Some G1 point is zero"
        );

        assert_eq!(self.g1s[0], G1Affine::generator(), "Expected G1 generator");
        assert_eq!(self.g2s[0], G2Affine::generator(), "Expected G2 generator");

        assert_ne!(self.g2s[1], G2Affine::identity(), "Scaled G2 point is zero");
        assert_ne!(self.g2s[1], self.g2s[0], "Scaled G2 point is the generator");

        // Check that the SRS has the correct structure. Instead of doing N individual
        // pairing checks, batch the G1 points via a random linear combination and do
        // only one pairing check
        let r_powers = powers(&Scalar::random(OsRng), self.g1s.len() - 1);
        let batched_lhs_g1 = msm_best(&r_powers, &self.g1s[..self.g1s.len() - 1]).to_affine();
        let batched_rhs_g1 = msm_best(&r_powers, &self.g1s[1..]).to_affine();

        assert_eq!(
            pairing(&batched_lhs_g1, &self.g2s[1]),
            pairing(&batched_rhs_g1, &self.g2s[0])
        )
    }

    /// Updates the given SRS (mutating it) with the given toxic waste `nu`,
    /// returns a proof of validity of the update
    pub fn update(&mut self, nu: &Scalar) -> UpdateProof {
        let n = self.g1s.len();
        let pb = initialize_progress_bar(n, Some(String::from("Adding randomness to the SRS")));

        let old_g1_point = self.g1s[1];

        // Update G1 points with fresh random scalar and compute
        // [nu * tau]_1, [nu^2 * tau^2]_1, ..., [nu^{N-1} * tau^{N-1}]_1
        self.g1s
            .par_iter_mut()
            .zip(powers(nu, n).par_iter())
            .inspect(|_| pb.inc(1))
            .for_each(|(point, power)| {
                *point = (*point * power).to_affine();
            });

        pb.finish_and_clear();

        self.g2s[1] = (self.g2s[1] * nu).to_affine();

        UpdateProof::create(old_g1_point, self.g1s[1], nu)
    }
}

// (De-)Serialization functionality
impl SRS {
    pub fn write_to_file(&self, path: &Path) {
        let mut file = create_file(path);

        for g1_point in &self.g1s {
            file.write_all(&g1_point.to_raw_bytes())
                .expect("Cannot write to file");
        }

        file.write_all(&self.g2s[0].to_raw_bytes())
            .expect("Cannot write to file");
        file.write_all(&self.g2s[1].to_raw_bytes())
            .expect("Cannot write to file");
    }

    pub fn read_from_file(path: &Path) -> Self {
        let mut file = open_file(path);
        let mut bytes = Vec::<u8>::new();
        file.read_to_end(&mut bytes).expect("Cannot read to end");

        let offset = bytes.len() - 2 * G2_SIZE;
        let pb = initialize_progress_bar(
            offset / G1_SIZE,
            Some(String::from("Reading the existing SRS")),
        );
        let g1s: Vec<G1Affine> = bytes[..offset]
            .par_chunks(G1_SIZE)
            .inspect(|_| pb.inc(1))
            .map(read_g1_point)
            .collect::<Vec<_>>();

        pb.finish_and_clear();

        let mut g2s = [G2Affine::generator(); 2];
        g2s[0] = read_g2_point(&bytes[offset..offset + G2_SIZE]);
        g2s[1] = read_g2_point(&bytes[offset + G2_SIZE..offset + 2 * G2_SIZE]);

        Self { g1s, g2s }
    }
}

#[cfg(test)]
mod srs_tests {
    use std::path::Path;

    use blstrs::{pairing, G1Affine, G2Affine, Scalar};
    use halo2curves::{
        ff::Field,
        group::{prime::PrimeCurveAffine, Curve},
    };
    use rand_core::{OsRng, RngCore};
    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

    use crate::{
        ceremony::{G1_SIZE, SRS},
        utils::{powers, read_g1_point_from_file},
    };

    #[cfg(test)]
    impl SRS {
        /// ONLY FOR TESTS
        ///
        /// Generate a random SRS of length n:
        /// [1]_1, [tau]_1,..., [tau^{n-1}]_1
        /// [1]_2, [tau]_2
        fn generate<R: RngCore>(n: usize, rng: R) -> Self {
            let tau = Scalar::random(rng);

            // Scalar powers: 1, tau, tau^2,..., tau^{n-1}
            let tau_powers = powers(&tau, n);

            // G1 points [tau]_1, ..., [tau^{n-1}]_1
            let g1s: Vec<G1Affine> = tau_powers
                .par_iter()
                .map(|power| (G1Affine::generator() * power).to_affine())
                .collect();

            let mut g2s = [G2Affine::generator(); 2];
            g2s[1] = (G2Affine::generator() * tau).to_affine();

            Self { g1s, g2s }
        }
    }

    #[test]
    fn generate_srs() {
        let srs = SRS::generate(1 << 12, OsRng);
        srs.verify_structure();

        let path = Path::new("/tmp/test");
        srs.write_to_file(path);

        let srs_deser = SRS::read_from_file(path);
        srs_deser.verify_structure();
    }

    #[test]
    fn generate_srs_with_update() {
        let mut srs = SRS::generate(1 << 10, OsRng);
        srs.verify_structure();
        let path = Path::new("/tmp/test_update");
        srs.write_to_file(path);

        let nu = Scalar::random(OsRng);
        let update_proof = srs.update(&nu);

        srs.verify_structure();

        let old_g1_point = read_g1_point_from_file(path, G1_SIZE);
        assert_eq!(old_g1_point, update_proof.g);
        update_proof.verify()
    }

    #[test]
    #[should_panic]
    fn srs_with_wrong_g1s_case1() {
        let mut srs = SRS::generate(1 << 10, OsRng);
        let k = OsRng.next_u64() % (srs.g1s.len() as u64);
        srs.g1s[k as usize] = G1Affine::identity();
        srs.verify_structure()
    }

    #[test]
    #[should_panic]
    fn srs_with_wrong_g1s_case2() {
        let mut srs = SRS::generate(1 << 12, OsRng);
        srs.g1s[1] = G1Affine::generator();
        srs.verify_structure();
    }

    #[test]
    #[should_panic]
    fn srs_with_wrong_g2s_case1() {
        let mut srs = SRS::generate(1 << 10, OsRng);
        srs.g2s[1] = G2Affine::identity();
        srs.verify_structure()
    }

    #[test]
    #[should_panic]
    fn srs_with_wrong_g2s_case2() {
        let mut srs = SRS::generate(1 << 10, OsRng);
        srs.g2s[1] = G2Affine::generator();
        srs.verify_structure()
    }

    #[test]
    fn malicious_pairing_checks() {
        let rng = OsRng;
        let false_lhs_1 = pairing(
            &G1Affine::identity(),
            &(G2Affine::generator() * Scalar::random(rng)).to_affine(),
        );
        let false_rhs_1 = pairing(&G1Affine::identity(), &G2Affine::generator());
        dbg!("Rejection if G1 point is 0: {}", false_lhs_1 == false_rhs_1);

        let false_lhs_2 = pairing(
            &(G1Affine::generator() * Scalar::random(rng)).to_affine(),
            &G2Affine::identity(),
        );
        let false_rhs_2 = pairing(
            &(G1Affine::generator() * Scalar::random(rng)).to_affine(),
            &G2Affine::identity(),
        );
        let false_rhs_3 = pairing(&G1Affine::identity(), &G2Affine::generator());
        dbg!(
            "Rejection if G1 or G2 point is 0: {}",
            false_lhs_2 == false_rhs_2 && false_lhs_2 == false_rhs_3
        );

        assert_eq!(false_lhs_1, false_rhs_1);
        assert_eq!(false_lhs_2, false_rhs_2);
    }
}

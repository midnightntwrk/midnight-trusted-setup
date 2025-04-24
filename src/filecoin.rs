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
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use blstrs::{G1Projective, Scalar};
use halo2curves::{
    ff::{Field, PrimeField},
    fft::best_fft,
    group::Curve,
    serde::SerdeObject,
};

use crate::{
    ceremony::{G1_SIZE, G2_SIZE},
    utils::{create_file, open_file, read_g1_point},
};

/// Converts Filecoin SRS from evaluation form to coefficient form
pub fn extract_g1_point_from_filecoin_srs(path: &Path, k: usize) {
    let mut file = open_file(path);

    // Read the phase1radix2m19 file, the result of running the following script:
    // https://github.com/filecoin-project/powersoftau/blob/ab8f85c28f04af5a99cfcc93a3b1f74c06f94105/src/bin/create_lagrange.rs
    // The first three elements correspond to:
    //
    // * [alpha]_1
    // * [beta]_1
    // * [beta]_2
    //
    // We are only interested in [tau]_1, so we ignore these three.

    let nr_powers = 1 << k;
    let offset: u64 = (G1_SIZE + G1_SIZE + G2_SIZE) as u64;
    file.seek(SeekFrom::Start(offset)).unwrap();

    println!("Parsing phase1radix2m19 file");
    let mut g1s: Vec<G1Projective> = Vec::<G1Projective>::with_capacity(nr_powers);
    let mut bytes = [0u8; G1_SIZE];
    for _ in 0..nr_powers {
        file.read_exact(&mut bytes).unwrap();
        g1s.push(read_g1_point(&bytes).into());
    }

    assert_eq!(
        nr_powers,
        g1s.len(),
        "# of read G1 points doesn't match # of expected points"
    );

    println!("Converting G1 points from eval form --> coeff form");
    let omega = Scalar::ROOT_OF_UNITY.pow([1 << (Scalar::S - k as u32) as u64]);
    best_fft(&mut g1s, omega, k as u32);

    let g1_point = g1s[1].to_affine();

    let mut file = create_file(Path::new("./filecoin_srs_g1_point"));
    g1_point
        .write_raw(&mut file)
        .expect("Could not write to file");
}

#[cfg(test)]
mod srs_tests {
    use std::{io::Read, path::Path};

    use crate::{
        ceremony::{G1_SIZE, G2_SIZE},
        utils::open_file,
    };

    // In order to run this test case, it is required to have the file
    // 'phase1radix2m19' in the project's root directory
    #[test]
    #[ignore = "This test requires having downloaded the phase1radix2m19 file"]
    fn test_phase1radix2m19_byte_structure() {
        let path = Path::new("./phase1radix2m19");
        let mut file = open_file(path);

        let mut buffer: Vec<u8> = Vec::new();
        let _ = file.read_to_end(&mut buffer);

        let nr_g1_points = 1 << 19;
        let byte_size_g1_points = nr_g1_points * G1_SIZE;

        let offset = G1_SIZE + G1_SIZE + G2_SIZE;
        let nr_g2_points = 1 << 19;
        let byte_size_g2_points = nr_g2_points * G2_SIZE;
        let expected_size = offset + 6 * (1 << 19) * G1_SIZE - G1_SIZE;

        dbg!("Size of header in bytes: {}", offset);
        dbg!("Expected nr of bytes: {}", expected_size);
        dbg!("Buffer size in bytes: {}", buffer.len());
        let balance = buffer.len()
            - (offset
                + byte_size_g1_points
                + byte_size_g2_points
                + byte_size_g1_points
                + byte_size_g1_points
                + (1 << 19) * G1_SIZE
                - G1_SIZE);
        dbg!("Difference: {}", balance);
        assert_eq!(expected_size, buffer.len())
    }
}

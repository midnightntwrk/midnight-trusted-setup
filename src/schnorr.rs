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

use blake2::Blake2b512;
use blstrs::{G1Affine, Scalar};
use halo2curves::{
    ff::{Field, FromUniformBytes},
    group::Curve,
    serde::SerdeObject,
};
use rand_core::OsRng;

use crate::{
    ceremony::{G1_SIZE, SCALAR_SIZE},
    utils::{create_file, hash_points, open_file, read_g1_point},
};

#[derive(Clone, Debug)]
pub struct SchnorrProof(G1Affine, Scalar);

impl SchnorrProof {
    /// Create a proof of knowledge of x such that x * G = H
    pub fn prove(g: G1Affine, h: G1Affine, x: &Scalar) -> Self {
        let r = Scalar::random(OsRng);
        let a = (g * r).to_affine();

        let e = Scalar::from_uniform_bytes(&hash_points::<Blake2b512>(&[g, h, a]));

        let z = r + x * e;
        SchnorrProof(a, z)
    }

    /// Verify a proof of knowledge of the dlog of H in base G; panics if the
    /// proof is not accepted
    pub fn verify(&self, g: G1Affine, h: G1Affine) {
        let (a, z) = (self.0, self.1);
        let e = Scalar::from_uniform_bytes(&hash_points::<Blake2b512>(&[g, h, a]));
        assert_eq!(g * z, h * e + a)
    }
}

#[derive(Clone, Debug)]
/// An update proof is a proof of knowledge of the dlog of h in base g, where
/// g is [tau]_1 of the previous SRS and h is [tau']_1 of the new SRS
pub struct UpdateProof {
    pub(crate) g: G1Affine,
    pub(crate) h: G1Affine,
    schnorr_proof: SchnorrProof,
}

impl UpdateProof {
    pub fn create(g: G1Affine, h: G1Affine, x: &Scalar) -> Self {
        UpdateProof {
            schnorr_proof: SchnorrProof::prove(g, h, x),
            g,
            h,
        }
    }

    pub fn verify(&self) {
        self.schnorr_proof.verify(self.g, self.h)
    }
}

// (De-)Serialization functionality
impl UpdateProof {
    pub fn write_to_file(&self, path: &Path) {
        let mut bytes = self.schnorr_proof.0.to_raw_bytes();
        bytes.extend(self.schnorr_proof.1.to_bytes_be());
        bytes.extend(self.g.to_raw_bytes());
        bytes.extend(self.h.to_raw_bytes());

        let mut file = create_file(path);
        file.write_all(&bytes)
            .expect("Could not write update proof to file");
    }

    pub fn read_from_file(path: &Path) -> Self {
        let mut file = open_file(path);
        let mut point_buf = [0u8; G1_SIZE];
        let mut scalar_buf = [0u8; SCALAR_SIZE];

        file.read_exact(&mut point_buf).expect("Not enough bytes");
        let schnorr_point = read_g1_point(&point_buf);

        file.read_exact(&mut scalar_buf).expect("Not enough bytes");
        let schnorr_scalar = Scalar::from_bytes_be(&scalar_buf)
            .expect("Failed to deserialize scalar of Schnorr proof");

        file.read_exact(&mut point_buf).expect("Not enough bytes");
        let g = read_g1_point(&point_buf);

        file.read_exact(&mut point_buf).expect("Not enough bytes");
        let h = read_g1_point(&point_buf);

        Self {
            schnorr_proof: SchnorrProof(schnorr_point, schnorr_scalar),
            g,
            h,
        }
    }
}

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
    fs::{self, DirEntry, File, ReadDir},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use blake2::{digest::consts::U64, Blake2b512, Digest};
use blstrs::{G1Affine, G2Affine, Scalar};
use halo2curves::{ff::Field, serde::SerdeObject};
use indicatif::{ProgressBar, ProgressStyle};
use rand_chacha::ChaCha20Rng;
use rand_core::{CryptoRng, RngCore, SeedableRng};

use crate::ceremony::G1_SIZE;

/// Opens the file at the given path, panics if something goes wrong
pub fn open_file(path: &Path) -> File {
    File::open(path).unwrap_or_else(|err| panic!("Failed to open file '{:?}': {}", path, err))
}

/// Creates a file at the given path, panics if something goes wrong
pub fn create_file(path: &Path) -> File {
    File::create(path).unwrap_or_else(|err| panic!("Failed to create file '{:?}': {}", path, err))
}

/// Opens the directory at the given path, panics if something goes wrong
pub fn open_dir(path: &Path) -> ReadDir {
    fs::read_dir(path).unwrap_or_else(|err| panic!("Failed to open dir '{:?}': {}", path, err))
}

/// Read a G1 point from the given buffer, panics if something goes wrong
pub fn read_g1_point(bytes: &[u8]) -> G1Affine {
    G1Affine::from_raw_bytes(bytes).expect("Failed to read G1 point")
}

/// Read a G2 point from the given buffer, panics if something goes wrong
pub fn read_g2_point(bytes: &[u8]) -> G2Affine {
    G2Affine::from_raw_bytes(bytes).expect("Failed to read G2 point")
}

/// Reads a G1 point from the given file after skipping `offset` bytes, panics
/// if something goes wrong
pub fn read_g1_point_from_file(path: &Path, offset: usize) -> G1Affine {
    let mut file = open_file(path);

    file.seek(SeekFrom::Start(offset as u64)).unwrap();
    let mut bytes = [0u8; G1_SIZE];
    file.read_exact(&mut bytes).expect("Invalid read exact");

    read_g1_point(&bytes)
}

/// Returns n powers of the given scalar: 1, s, s^2, ..., s^(n-1)
pub fn powers(s: &Scalar, n: usize) -> Vec<Scalar> {
    std::iter::successors(Some(Scalar::ONE), |p| Some(*p * s))
        .take(n)
        .collect()
}

/// Hashes (with the specified hash function) the given slice of points
pub fn hash_points<H>(points: &[G1Affine]) -> [u8; 64]
where
    H: Digest<OutputSize = U64>,
{
    let mut hasher = H::new();
    for p in points {
        hasher.update(p.to_raw_bytes());
    }
    hasher.finalize().into()
}

/// Initialize progress bar for display progress of verifying and updating SRS
pub fn initialize_progress_bar(nr_points: usize, msg: Option<String>) -> ProgressBar {
    let pb = ProgressBar::new(nr_points as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {percent}% {msg}")
            .unwrap()
            .progress_chars("#-"),
    );
    if let Some(msg) = msg {
        pb.set_message(msg);
    }
    pb
}

/// Open all update proof directories from the default folder; return a vector
/// of them sorted by the canonical order
pub fn open_update_proof_dirs() -> Vec<DirEntry> {
    let path = Path::new("./proofs");
    let mut proof_files: Vec<(usize, DirEntry)> = Vec::new();
    for entry in open_dir(path) {
        let entry = entry.expect("Invalid proof file");
        let file_name = entry
            .file_name()
            .into_string()
            .expect("Failed to parse canonical file name");

        if let Some(number) = file_name
            .strip_prefix("proof")
            .and_then(|s| s.parse::<usize>().ok())
        {
            proof_files.push((number, entry));
        }
    }

    // Sort files by extracted number
    proof_files.sort_by_key(|&(num, _)| num);

    proof_files.into_iter().map(|(_, dir)| dir).collect()
}

/// Create path for new SRS file based on previous number of updates
pub fn derive_new_path(old_path: &Path) -> (PathBuf, PathBuf) {
    let proofs_path = Path::new("proofs/");

    let n = open_dir(proofs_path).filter_map(|entry| entry.ok()).count() + 1;

    let new_srs_path = old_path.parent().unwrap().join(format!("srs{n}"));
    let new_proof_path = proofs_path.join(format!("proof{n}"));

    (new_srs_path, new_proof_path)
}

/// Generates a scalar from various randomness sources
pub fn generate_toxic_waste(mut rng: impl RngCore + CryptoRng) -> Scalar {
    // Use Blake2b for combining output from different entropy sources
    let mut hasher = Blake2b512::new();

    // Read random user input
    let mut user_input = String::new();
    println!("\nPlease, hit your keyboard randomly then press [ENTER]. (This will not be the only source of entropy.)");
    std::io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read user input");
    hasher.update(user_input);

    // In addition, get some random bytes from the OS
    let mut os_input = [0u8; 512];
    rng.try_fill_bytes(&mut os_input)
        .expect("Could not fill bytes");
    hasher.update(os_input);

    // Hash it all together and use hash as seed for RNG
    let digest: [u8; 32] = hasher.finalize()[0..32].try_into().unwrap();

    Scalar::random(ChaCha20Rng::from_seed(digest))
}

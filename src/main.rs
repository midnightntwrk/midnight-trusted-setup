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

use std::path::Path;

use clap::{Parser, Subcommand};
use rand_core::OsRng;

mod schnorr;
use schnorr::UpdateProof;

mod ceremony;
use ceremony::{G1_SIZE, SRS};

mod utils;
use utils::{
    derive_new_path, generate_toxic_waste, open_update_proof_dirs, read_g1_point_from_file,
};

mod filecoin;
use filecoin::extract_g1_point_from_filecoin_srs;

// Struct to represent command-line arguments
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct CLICommand {
    #[command(subcommand)]
    cmd: Command,
    srs_path: String,
}

#[derive(Subcommand, Debug)]
enum Command {
    VerifyStructure {
        /// Asserting 2**log2_len G1 elements in the SRS (incl. the generator)
        #[arg(short, long)]
        log2_len: usize,
    },
    VerifyChain,
    Update,
    ExtractFilecoinG1Point,
}

fn verify_chain(last_srs_path: &Path) {
    println!("\nVerifying the chain of update proofs...");

    let first_g1_point = read_g1_point_from_file(Path::new("./filecoin_srs_g1_point"), 0);
    let last_g1_point = read_g1_point_from_file(last_srs_path, G1_SIZE);

    let chain_of_proofs: Vec<UpdateProof> = open_update_proof_dirs()
        .iter()
        .map(|e| UpdateProof::read_from_file(&e.path()))
        .collect();

    let mut g = first_g1_point;
    for proof in chain_of_proofs {
        assert_eq!(proof.g, g);
        assert_ne!(proof.g, proof.h);
        proof.verify();
        g = proof.h;
    }

    assert_eq!(g, last_g1_point);

    println!("The chain of update proofs is correct!\n");
}

fn update(old_srs_path: &Path) {
    println!("\nRe-randomizing the existing SRS...");

    let (new_srs_path, new_proof_path) = derive_new_path(old_srs_path);

    let nu = generate_toxic_waste(OsRng);

    let mut srs = SRS::read_from_file(old_srs_path);

    // Check that current_g = previous_h
    // I.e., the current update correctly extends the previous update
    assert_eq!(
        srs.g1s[1],
        UpdateProof::read_from_file(&open_update_proof_dirs().last().unwrap().path()).h,
        "SRS doesn't match chain of updates"
    );

    let proof = srs.update(&nu);

    print!("Writing the SRS to file...");
    srs.write_to_file(&new_srs_path);
    proof.write_to_file(&new_proof_path);

    println!(
        "\rThank you for your participation!\n\nThe SRS in {:?} has been successfully updated and saved to {:?}.\n",
        old_srs_path.canonicalize().unwrap(),
        new_srs_path.canonicalize().unwrap()
    );

    println!(
        "Make sure you upload your updated SRS to the SFTP server and open a PR with your validity proof (saved at {:?}).\n",
        new_proof_path.canonicalize().unwrap()
    );
}

fn verify_structure(srs_path: &Path, length: usize) {
    println!("\nVerifying structure of the SRS...");

    let srs = SRS::read_from_file(srs_path);

    let expected_len = 1 << length;
    assert_eq!(
        srs.g1s.len(),
        expected_len,
        "Expected {} elements in G1, but found {}.",
        expected_len,
        srs.g1s.len(),
    );

    srs.verify_structure();

    println!(
        "The structure of the SRS in {:?} is correct!\n",
        srs_path.canonicalize().unwrap()
    )
}

fn extract(phase1radix_path: &Path) {
    extract_g1_point_from_filecoin_srs(phase1radix_path, 19);

    println!(
        "First G1 point succesfully extracted from {:?}!\n",
        phase1radix_path.canonicalize().unwrap()
    )
}

fn main() {
    let args = CLICommand::parse();

    match args.cmd {
        Command::VerifyStructure { log2_len } => {
            verify_structure(Path::new(&args.srs_path), log2_len)
        }
        Command::VerifyChain => verify_chain(Path::new(&args.srs_path)),
        Command::Update => update(Path::new(&args.srs_path)),
        Command::ExtractFilecoinG1Point => extract(Path::new(&args.srs_path)),
    };

    println!(
        "
▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓       ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
▓▓▓▓▓▓▓▓▓▓   ▓▓▓▓▓▓▓▓▓▓▓   ▓▓▓▓▓▓▓▓▓▓
▓▓▓▓▓▓▓   ▓▓▓▓▓▓▓   ▓▓▓▓▓▓▓   ▓▓▓▓▓▓▓
▓▓▓▓▓   ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓   ▓▓▓▓▓
▓▓▓▓   ▓▓▓▓▓▓▓▓▓▓   ▓▓▓▓▓▓▓▓▓▓   ▓▓▓▓
▓▓▓   ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓   ▓▓▓
▓▓▓   ▓▓▓▓▓▓▓▓▓▓▓   ▓▓▓▓▓▓▓▓▓▓▓   ▓▓▓
▓▓▓   ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓   ▓▓▓
▓▓▓▓   ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓   ▓▓▓▓
▓▓▓▓▓   ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓   ▓▓▓▓▓
▓▓▓▓▓▓▓   ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓   ▓▓▓▓▓▓▓
▓▓▓▓▓▓▓▓▓▓   ▓▓▓▓▓▓▓▓▓▓▓   ▓▓▓▓▓▓▓▓▓▓
▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓       ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓"
    );
}

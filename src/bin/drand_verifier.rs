//! `drand` Verifier - Verifies that an SRS update was created using `drand`
//! randomness
//!
//! This tool verifies that the last SRS update in the ceremony was created
//! using randomness from a specific committed round of `drand`, providing
//! public verifiability.
//!
//! # How it works
//!
//! 1. Verifies the commitment matches Blake2b-512(round || salt)
//! 2. Fetches the drand signature for the specified round from the drand API
//! 3. Verifies the drand signature is cryptographically valid
//! 4. Derives the scalar using the same process as the update:
//!    - Calls [derive_randomness] to extract randomness from the signature
//!    - Computes `seed = Blake2b-512(derive_randomness(signature) ||
//!      salt)[0..32]`
//!    - Generates `scalar = Scalar::random(ChaCha20Rng::from_seed(seed))`
//! 5. Reads the last update proof and verifies that `proof.h == proof.g *
//!    scalar`
//!
//! If all checks pass, this proves the last SRS update was created using the
//! committed `drand` round and `salt` used in the commitment to this round.

use blake2::{Blake2b512, Digest};
use blstrs::Scalar;
use clap::Parser;
use drand_verify::{derive_randomness, verify, G1Pubkey, Pubkey};
use halo2curves::{ff::Field, group::Curve};
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(name = "drand-verifier")]
#[command(
    about = "Verifies a (pre-committed) drand round and checks that the last SRS update correctly used the drand randomness as seed."
)]
#[command(
    long_about = "Verifies that an SRS update was created using randomness from a specific committed drand round.\n\n\
                         This tool fetches and verifies the drand signature for a given committed round, verifies the commitment to this round, derives the scalar using\n\
                         derive_randomness(signature) combined with the salt, and checks that the last\n\
                         update proof matches this scalar."
)]
struct Args {
    /// The drand round number used for the update
    #[arg(short, long)]
    round: u64,

    /// The salt (hex) used in the commitment (16 bytes recommended)
    #[arg(short, long)]
    salt: String,

    /// The commitment (hex) = Blake2b-512(round || salt)
    #[arg(short, long)]
    commitment: String,

    /// Additionally verify the entire drand chain from round 1 to the specified
    /// round
    #[arg(long, default_value_t = false)]
    verify_chain: bool,
}

#[derive(Debug, Deserialize)]
struct DrandResponse {
    #[allow(dead_code)]
    round: u64,
    signature: String,
    #[serde(default)]
    previous_signature: Option<String>,
}

// https://api.drand.sh/v2/beacons/default/info
const DRAND_PUBLIC_KEY: &str = "868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31";

fn fetch_drand_round(round: u64) -> Result<DrandResponse, std::io::Error> {
    ureq::get(&format!(
        "https://api.drand.sh/v2/beacons/default/rounds/{}",
        round
    ))
    .call()
    .map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Error in HTTPS call: {:?}", e),
        )
    })?
    .into_json()
}

fn verify_signature(round: u64, signature: &[u8], previous_signature: &[u8], public_key_hex: &str) {
    let pubkey = G1Pubkey::from_variable(&hex::decode(public_key_hex).unwrap()).unwrap();

    if !verify(&pubkey, round, previous_signature, signature).unwrap() {
        eprintln!("Signature verification of round {round} failed");
        std::process::exit(1);
    }
}

fn verify_chain(start_round: u64, end_round: u64) {
    let mut previous_sig: Vec<u8> = Vec::new();

    for round in start_round..=end_round {
        if round % 100 == 0 || round == end_round {
            println!("Round {}/{}", round, end_round);
        }

        let response = fetch_drand_round(round)
            .unwrap_or_else(|_| panic!("Failed to fetch DRAND round {round}."));
        let signature = hex::decode(&response.signature)
            .expect("Invalid signature format from DRAND response.");
        verify_signature(round, &signature, &previous_sig, DRAND_PUBLIC_KEY);
        previous_sig = signature;
    }
    println!("Chain successfully verified");
}

fn verify_commitment(round: u64, salt: &[u8], commitment: &[u8]) {
    let mut data = round.to_le_bytes().to_vec();
    data.extend_from_slice(salt);

    let hash = Blake2b512::digest(&data);
    if &hash[..] != commitment {
        eprintln!("Commitment verification failed");
        std::process::exit(1);
    }
}

fn main() {
    let args = Args::parse();

    let salt = hex::decode(&args.salt).expect("Failed to decode salt.");
    let commitment = hex::decode(&args.commitment).expect("Failed to decode commitment.");

    verify_commitment(args.round, &salt, &commitment);

    let drand_response = fetch_drand_round(args.round).expect("Failed to fetch drand round.");

    if args.verify_chain {
        verify_chain(1, args.round);
    }

    let signature = hex::decode(&drand_response.signature).expect("Failed to decode signature.");
    let previous_sig = drand_response
        .previous_signature
        .as_ref()
        .map(hex::decode)
        .transpose()
        .unwrap()
        .unwrap_or_default();

    verify_signature(args.round, &signature, &previous_sig, DRAND_PUBLIC_KEY);

    let seed = Blake2b512::new()
        .chain_update(derive_randomness(&signature))
        .chain_update(&salt)
        .finalize()[..32]
        .try_into()
        .unwrap();

    let scalar = Scalar::random(ChaCha20Rng::from_seed(seed));

    // We now take the last two contributions, and check that the last corresponds
    // to an update of the previous with the randomness above.

    println!(
        "Verifying that the last contribution was created with drand randomness from round {}...",
        args.round
    );

    // Read the last update proof and verify the scalar matches
    let update_proofs = srs::utils::open_update_proof_dirs();
    if update_proofs.len() < 2 {
        eprintln!("Need at least 2 update proofs to verify the last contribution");
        std::process::exit(1);
    }

    let last_proof =
        srs::schnorr::UpdateProof::read_from_file(&update_proofs.last().unwrap().path());

    // Verify that h = g * scalar (i.e., the last update used our scalar)
    if (last_proof.g * scalar).to_affine() != last_proof.h {
        eprintln!("Verification failed: The last contribution does not match the drand randomness");
        std::process::exit(1);
    }

    println!("Verification successful! The last contribution was correctly generated using round {} of drand", args.round);
}

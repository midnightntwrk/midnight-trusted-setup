//! Drand Verifier - Verifies that an SRS update was created using Drand
//! randomness.
//!
//! This tool verifies that the last SRS update in the ceremony was created
//! using randomness from a specific committed round of Drand, providing
//! public verifiability.
//!
//! # How it works
//!
//! 1. Verifies the commitment matches SHA-256(round || salt)
//! 2. Fetches the Drand signature for the specified round from the Drand API
//! 3. Verifies the Drand signature is cryptographically valid
//! 4. Derives the scalar using the same process as the update:
//!    - Calls [derive_randomness] to extract randomness from the signature
//!    - Computes `seed = Blake2b-512(randomness || salt)`
//!    - Generates `scalar = Scalar::random(ChaCha20Rng::from_seed(seed))`
//! 5. Reads the last update proof and verifies that `proof.h == proof.g *
//!    scalar`
//!
//! If all checks pass, this proves the last SRS update was created using the
//! randomness form the committed Drand round and the `salt` used in for such
//! commitment.

use blake2::{Blake2b512, Digest};
use blstrs::Scalar;
use clap::Parser;
use drand_verify::{derive_randomness, verify, G1Pubkey, Pubkey};
use halo2curves::{ff::Field, group::Curve};
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use serde::Deserialize;
use sha2::Sha256;

#[derive(Parser, Debug)]
#[command(name = "drand-verifier")]
#[command(
    about = "Verifies a (pre-committed) Drand round and checks that the last SRS update correctly used the Drand randomness as seed."
)]
#[command(
    long_about = "Verifies that an SRS update was created using randomness from a specific committed Drand round.\n\n\
                  This tool fetches and verifies the Drand signature for a given committed round, verifies the commitment to this round, derives the scalar using\n\
                  derive_randomness(signature) combined with the salt, and checks that the last\n\
                  update proof matches this scalar."
)]
struct Args {
    /// The Drand round number used for the update
    #[arg(short, long)]
    round: u64,

    /// The salt (hex) used in the commitment to the round number (16 bytes)
    #[arg(short, long)]
    salt: String,

    /// The commitment (hex) to the round number, supposedly
    /// SHA-256(round || salt)
    #[arg(short, long)]
    commitment: String,
}

#[derive(Debug, Deserialize)]
struct DrandResponse {
    #[allow(dead_code)]
    round: u64,
    signature: String,
    #[serde(default)]
    previous_signature: Option<String>,
}

/// https://api.drand.sh/v2/beacons/default/info
const DRAND_PUBLIC_KEY: &str = "868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31";

/// Fetches the Drand information, for the given round number, from the public
/// Drand API. This information includes the round signature and the previous
/// signature.
fn fetch_drand_round(round: u64) -> Result<DrandResponse, std::io::Error> {
    ureq::get(&format!(
        "https://api.drand.sh/v2/beacons/default/rounds/{}",
        round
    ))
    .call()
    .map_err(|e| std::io::Error::other(format!("Error in HTTPS call: {:?}", e)))?
    .into_json()
}

/// Verifies the Drand signature for the given round.
fn verify_signature(round: u64, signature: &[u8], previous_signature: &[u8], public_key_hex: &str) {
    let pubkey = G1Pubkey::from_variable(&hex::decode(public_key_hex).unwrap()).unwrap();

    assert!(
        verify(&pubkey, round, previous_signature, signature).unwrap(),
        "Signature verification of round {round} failed."
    );
}

/// Verify that `commitment` opens to `round || salt`.
///
/// Namely, assert that `commitment == SHA-256(round || salt)`,
/// where `round` is encoded as 16 bytes in little-endian.
fn verify_commitment(round: u64, salt: &[u8; 16], commitment: &[u8]) {
    let mut data = round.to_le_bytes().to_vec();
    data.resize(16, 0);
    data.extend_from_slice(salt);

    let hash = Sha256::digest(&data);

    assert_eq!(&hash[..], commitment, "Commitment verification failed.");
}

fn main() {
    let args = Args::parse();

    let mut salt = [0u8; 16];
    hex::decode_to_slice(&args.salt, &mut salt).expect("Failed to decode salt.");

    let commitment = hex::decode(&args.commitment).expect("Failed to decode commitment.");

    verify_commitment(args.round, &salt, &commitment);
    print!(
        "Commitment successfully verified!\nSHA-256({}u64 || {}) = {}\n\n",
        args.round, args.salt, args.commitment,
    );

    let drand_response = fetch_drand_round(args.round).expect("Failed to fetch Drand round.");

    let signature = hex::decode(&drand_response.signature).expect("Failed to decode signature.");
    let previous_sig = drand_response
        .previous_signature
        .as_ref()
        .map(hex::decode)
        .transpose()
        .unwrap()
        .unwrap_or_default();

    verify_signature(args.round, &signature, &previous_sig, DRAND_PUBLIC_KEY);
    let round_randomness = derive_randomness(&signature);
    print!(
        "Drand round {} was fetched correctly, its signature is valid!\nThe round randomness is: {}\n\n",
        args.round,
        hex::encode(round_randomness)
    );

    // Compute the scalar exactly as in the update process, from the Drand
    // randomness, concatenated with the salt

    let mut buffer = String::new();
    buffer.push_str(&hex::encode(round_randomness));
    buffer.push_str(&hex::encode(salt));

    let mut hasher = Blake2b512::new();
    hasher.update(buffer);

    let seed: [u8; 32] = hasher.finalize()[0..32].try_into().unwrap();
    let scalar = Scalar::random(ChaCha20Rng::from_seed(seed));

    println!(
        "The scalar derived from the Drand round randomness and the provided salt is:\n{scalar}\n",
    );

    // We now take the last two contributions, and check that the last corresponds
    // to an update of the previous with the randomness above
    let update_proofs = srs::utils::open_update_proof_dirs();
    let last_update_proof_file = update_proofs.last().unwrap().path();
    let last_proof = srs::schnorr::UpdateProof::read_from_file(&last_update_proof_file);

    // Verify that h = g * scalar (i.e., the last update used our scalar)
    assert_eq!(
        (last_proof.g * scalar).to_affine(),
        last_proof.h,
        "The last contribution (proved in file {last_update_proof_file:?}) was NOT performed with the expected scalar"
    );

    println!(
        "The last contribution (proved in file {:?}) was performed with the expected scalar",
        last_update_proof_file
    );

    println!("\nAll checks passed!");
}

# Trusted Setup for PLONK over BLS12-381

This is the official repository for the trusted-setup ceremony of the
[Midnight Network](https://midnight.network/). The ceremony ended on
Dec 16 2025 (AoE).

The outcome of this ceremony is a so-called *powers-of-tau* structured
reference string (SRS) over the BLS12-381 elliptic curve of length $2^{25}$
(see our [wiki](WIKI.md) for more details on the type and length of the SRS).

This ceremony is performed on top of
[Filecoin SRS](https://trusted-setup.filecoin.io/phase1/), an existing
powers-of-tau SRS over BLS12-381 used on [Filecoin](https://filecoin.io/)
and trusted by the Web3 community.

During Midnight Network's ceremony, the Filecoin SRS has been updated
(i.e., re-randomized) multiple times by various 
[participants](https://github.com/midnightntwrk/midnight-trusted-setup/blob/main/PARTICIPANTS.md).
Each update generated not only a new SRS but also an update proof, which
guarantees that the new SRS has been computed by re-randomizing the previous
SRS (and not from scratch). The chain of update proofs is available in the 
[`proofs/`](https://github.com/midnightntwrk/midnight-trusted-setup/tree/main/proofs)
directory of this repository.

> [!IMPORTANT]
> This ceremony ended on Dec 16 2025 (AoE), no more contributions are accepted.

## Midnight SRS

The resulting SRS from this ceremony is available at [Midnight SRS], here are
its details:
```
Official URL:   https://srs.midnight.network/current_srs/powers_of_tau
   File size:   3,221,225,856 bytes
  SHA256 sum:   df7a1e9fcd6d3f6e8ddd777914c40c44cd29777b769e608c0604fbfbe83121ce
```

Find instructions [below](#verify-the-midnight-srs) on how to verify its
validity.

Extended SRS files (containing both coefficient and Lagrange representations)
derived from this official ceremony output are cataloged in
[`MIDNIGHT_SRS_CATALOG.md`](MIDNIGHT_SRS_CATALOG.md).

## Build the CLI Tool

After cloning the repository, build it, and copy the binary to the root
folder of the repository with the commands:
```sh
cargo build --release 
cp ./target/release/srs_utils ./target/release/drand_verifier ./
```

## Verify the Midnight SRS
Anyone can verify the integrity of the Midnight SRS (please note the
[hardware requirements](#hardware-requirements)).
We invite any user of Midnight (and also the wider public) to do so.

0. If you have not done so already, [build](#build-the-cli-tool) the official
   CLI tool.

1. Download the [Midnight SRS] (its size is about `3.2 GB`).

2. Compute the SHA256 sum of the downloaded SRS, and compare it with the
   [checksum](#midnight-srs) above.

3. Verify the structural integrity of the SRS, and that it has the expected
   length of $2^{25}$:
   ```sh
   ./srs_utils <PATH-TO-MIDNIGHT-SRS> verify-structure -l 25
   ```

4. Verify the chain of update proofs that links Midnight's SRS to
   Filecoin's SRS:
   ```sh
   ./srs_utils <PATH-TO-MIDNIGHT-SRS> verify-chain
   ```

5. Verify that the final update was performed as declared in the
   [end-of-ceremony section](#end-of-the-srs-ceremony). Find instructions
   [below](#verification-of-the-last-iteration).

6. (Optional). The chain of update proofs starts at Filecoin's
   [G1 point](filecoin_srs_g1_point). See our [wiki](WIKI.md) for details on
   how to verify the validity of this point.


## Participate (Closed on Dec 16, 2025 - AoE)

### Prerequisites

* A GitHub account, with a linked SSH key, and set up with signed commits.
* A working Rust installation.

### Hardware requirements

A machine with at least 8GB of RAM is required.

### Instructions

1. Open a GitHub issue in this repository using the [Request to Participate in SRS
   Ceremony](https://github.com/midnightntwrk/midnight-trusted-setup/issues/new?template=request-participation.md)
   template to request a participation slot. You will be assigned a participation
   number `N` and will be notified via GitHub when your turn arrives.

2. On your turn, download the [Midnight SRS]. You can optionally verify its
   integrity by computing its SHA256 digest, comparing this value with the
   corresponding checksum in `PARTICIPANTS.md`, and by running steps
   [3., 4., and 6. above](#verify-the-midnight-srs).

3. Re-randomize it with `./srs_utils <PATH-TO-DOWNLOADED-SRS> update`. This
   process will create 2 files: a file named `srs<N>` in the same location
   where you stored the downloaded SRS and a file named `proof<N>` in the
   `proofs/` directory.

4. Compute the SHA256 digest of the updated SRS, e.g., with
   `sha256sum <PATH-TO-SRS-N>`. Add a new row to `PARTICIPANTS.md` with
   your name, GitHub handle, affiliation (optional), and the SHA256 digest.

5. Open a pull request with the `proof<N>` and `PARTICIPANTS.md` file (please
   make sure your GH account uses signed commits).

6. Upload the `srs<N>` file to the internet (we provide a server for this
   purpose if it helps you, see the
   [instructions](#upload-the-srs-to-our-server) below).

7. After we verify that your SRS is structurally correct and your update
   proof correctly extends the chain of proofs, your PR will be merged and
   the next participant will take over.

> [!WARNING]
> A **24-hour completion window** is enforced for submissions to maintain ceremony
> liveness. Unfinished submissions will be declined, and the next participant invited.
> This is a liveness measure, **NOT censorship.** (Note that running 
> the update script itself only takes a few minutes on a commodity laptop.)

Depending on the prerequisites of the participant, setting up a GitHub
account and a Rust installation may take some time as well.

### Upload the SRS to our server

You may use our server to upload your updated SRS via SFTP. Simply run:

```sh
sftp -v <YOUR_GITHUB_USERNAME>@sftp.trusted-setup.midnight.network
put <PATH-TO-UPDATED-SRS> .
```

## End of the SRS ceremony

The ceremony ended on Dec 16, 2025 (AoE). After that, one last additional
iteration was performed, using a randomness beacon as entropy source.
This was done to ensure that the final SRS is unbiased.

Concretely, the toxic waste used in the last iteration was derived with
`ChaCha20`, seeded with entropy from [Drand](https://github.com/drand/drand),
a distributed randomness beacon that produces 32 bytes of entropy every 30
seconds in so-called "rounds".

The Drand network produces publicly verifiable, unbiasable and 
unpredictable random values in a distributed manner using threshold 
cryptography. In particular, the random values provided by Drand stay 
unbiased, even if an adversary controls more than the threshold number of 
nodes in the network. For more information, refer to their 
[docs](https://docs.drand.love/).

### How we selected the randomness

We used the random value `r` from a specific Drand round `N` to seed 
the last iteration of the ceremony. Such `N` was 
[declared on Dec 10, 2025](https://github.com/midnightntwrk/midnight-trusted-setup/commit/c1e700b299167db84e0b14c652a3bf8e8646b258)
(in committed form `C`) by hashing the concatenation of `N` (encoded as a
16-byte little-endian unsigned integer) with a 16-byte random `SALT`.

```
C = SHA256(N || SALT) = 4282753f1830effbef453338577e682ecb2714a0de4ecf4998546f18e314f7f3
```

On Dec 18, 2025, we declare the commitment opening:
```
   N = 5686659 (dec)
SALT = 620f6c7da172dc454ec2361dc0673407 (hex)
```

Note that Drand round `N = 5686659` was sampled on Dec 18, 2025 at around
4:06 am UTC, several hours after the last SRS contribution.

### How we seeded the last iteration

The following steps for seeding the last iteration (with the Dround entropy)
were designed and
[declared](https://github.com/midnightntwrk/midnight-trusted-setup/commit/c1e700b299167db84e0b14c652a3bf8e8646b258)
a week before the `5686659`-th Drand round was sampled.

The randomness `r` from the
[`5686659`-th Drand round](https://api.drand.sh/public/5686659) is:
```
r = d486b50013d1bb3fe95d1a303a485bb15fb617622b6cf253115cd540ed76a91b (hex)
```

The toxic waste `tau` of the last re-randomization of the SRS was
derived by seeding ChaCha20 with the Blake2b512 digest of `r || SALT`
(encoded as a hexadecimal string in ASCII), where `SALT` is the one we used
in the commitment `C` to `N` from above:

```
RNG = ChaCha20::from_seed(Blake2b512(hex::encode(r || SALT)))
tau = Scalar::random(RNG)
```

Because the commitment `C` was published in advance, and the Drand beacon 
is publicly verifiable, anyone will be able to independently verify the 
commitment opening `N || SALT`, retrieve the randomness `r`, recompute the 
toxic waste `tau`, and confirm that the final SRS update is correct.

This ensures a transparent, unbiased, and fully reproducible conclusion to 
the ceremony.

### Verification of the last iteration

You can verify the final update using the `drand_verifier` binary:
```sh
./drand_verifier \
  --round 5686659 \
  --salt 620f6c7da172dc454ec2361dc0673407 \
  --commitment 4282753f1830effbef453338577e682ecb2714a0de4ecf4998546f18e314f7f3
```

This command will:
1. Verify that commitment `C` correctly opens to `N`.
2. Fetch the data associated to the `5686659`-th Drand round and validate
   its signature.
3. Derive the scalar `tau` used in the last iteration, as detailed
   [above](#how-we-seed-the-last-iteration).
4. Make sure that the last contribution (proved in file
   [`./proofs/proof38`](https://github.com/midnightntwrk/midnight-trusted-setup/blob/main/proofs/proof38))
   was performed with such `tau`.

[Midnight SRS]: https://srs.midnight.network/current_srs/powers_of_tau

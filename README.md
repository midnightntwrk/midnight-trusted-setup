# Trusted Setup for PLONK over BLS12-381

This is the official repository for the trusted-setup ceremony of the
[Midnight Network](https://midnight.network/).

The outcome of this ceremony will be a so-called *powers-of-tau* structured
reference string (SRS) over the BLS12-381 elliptic curve of length $2^{25}$
(see our [wiki](WIKI.md) for more details on the type and length of the SRS).

The ceremony is based on the
[Filecoin ceremony](https://trusted-setup.filecoin.io/phase1/), an existing
powers-of-tau SRS over BLS12-381 used on [Filecoin](https://filecoin.io/)
and trusted by the Web3 community.

During Midnight Network's ceremony, the Filecoin SRS will be updated 
(i.e., re-randomized) multiple times by various participants. Each update
generates not  only a new SRS but also an update proof, which proves that
the previous SRS was used correctly in the update process. The chain of
update proofs will be stored in the `proofs/` directory of this repository.

> [!IMPORTANT]
> This ceremony is currently ongoing and you can be a participant!
> See the instructions below.

## Prerequisites

* A GitHub account, with a linked SSH key, and set up with signed commits.
* A working Rust installation.

## Hardware requirements
A machine with at least 8GB of RAM is required.

## Build the CLI

After cloning the repository, build it, and copy the binary to the root
folder of the repository with the commands:
```sh
cargo build --release 
cp ./target/release/srs_utils ./
```

## Participate

1. Open a GitHub issue in this repository using the [Request to Participate in SRS
   Ceremony](https://github.com/midnightntwrk/midnight-trusted-setup/issues/new?template=request-participation.md)
   template to request a participation slot. You will be assigned a participation number `N`
   and will be notified via github when your turn arrives.

2. On your turn, download the [LatestSRS]. You can optionally verify its
   structure and `sha256sum` as explained [below](#verify-the-latest-srs).

3. Re-randomize it with `./srs_utils <PATH-TO-DOWNLOADED-SRS> update`. This
   process will create 2 files: a file named `srs<N>` in the same location
   where you stored the downloaded SRS and a file named `proof<N>` in the
   `proofs/` directory.

4. Compute the SHA256 digest of the updated SRS, e.g. with
   `sha256sum <PATH-TO-SRS-N>`, and add a new row to `PARTICIPANTS.md` with
   your name, GitHub handle, affiliation (optional), and SHA256 digest.

5. Open a pull request with the `proof<N>` and `PARTICIPANTS.md` file (please
   make sure your GH account uses signed commits).

6. Upload the `srs<N>` file to the internet (we provide a server for this
   purpose if it helps you, see the
   [instructions](#upload-the-srs-to-our-server) below).

7. After we verify that your SRS is structurally correct and your update
   proof correctly extends the chain of proofs, your PR will be merged and
   the next participant will take over.

> [!WARNING]
>A **24-hour completion window** is enforced for submissions to maintain ceremony
> liveness. Unfinished submissions will be declined, and the next participant invited.
> This is a liveness measure, **NOT censorship.** (Note that running 
> the update script itself only takes a few minutes on a commodity laptop.)

Depending on the prerequisites of the participant, setting up a GitHub
account and and a Rust installation may take some time as well.

### Upload the SRS to our server

You may use our server to upload your updated SRS via SFTP. Simply run:

```sh
sh sftp -v <YOUR_GITHUB_USERNAME>@sftp.trusted-setup.midnight.network
put <PATH-TO-UPDATED-SRS> .
```

## Optional: Verify the latest SRS

The latest SRS can be found at [LatestSRS], its exact size is `3221225856 B`
(about `3.2 GB`). Once you have downloaded the latest SRS, you can confirm its
authenticity by comparing its SHA-256 checksum with the one listed in
PARTICIPANTS.md:
```sh
sha256sum <PATH-TO-DOWNLOADED-SRS>
```

Next, you may verify that it is structurally correct and has the expected
length of $2^{25}$ with:
```sh
./srs_utils <PATH-TO-DOWNLOADED-SRS> verify-structure -l 25
```

You can also verify the chain of update proofs that link the latest SRS to
Filecoin's SRS. Before this, please make sure you track the most recent version
of this repository in order to have all the necessary update proofs.

For verifying the chain, simply run:

```sh
./srs_utils <PATH-TO-DOWNLOADED-SRS> verify-chain
```

This chain starts at Filecoin's [G1 point](filecoin_srs_g1_point).
See our [wiki](WIKI.md) for details on how to verify the validity of this
point.

[LatestSRS]: https://srs.midnight.network/current_srs/powers_of_tau

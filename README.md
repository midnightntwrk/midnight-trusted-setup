# Trusted Setup for PLONK over BLS12-381

This is the official repository for the trusted-setup ceremony of the
[Midnight Network](https://midnight.network/).

The outcome of this ceremony will be a so-called *powers-of-tau* structured
reference string (SRS) over the BLS12-381 elliptic curve.

> [!IMPORTANT]
> This ceremony is currently ongoing and you can be a participant!
> See the [instructions](#participate) below.

The ceremony is based on the
[Filecoin ceremony](https://trusted-setup.filecoin.io/phase1/), an existing
powers-of-tau SRS over BLS12-381 used on [Filecoin](https://filecoin.io/)
and trusted by the Web3 community.

The Filecoin SRS will be re-randomized (or updated) several times by various
participants. In addition to a new SRS, each update also generates an update
proof attesting to the fact that the update process correctly used the previous
SRS.

The chain of update proofs will be stored in the `proofs/` directory of this
repository.

## Prerequisites

* A GitHub account, with a linked SSH key, and set up with signed commits.
* A working Rust installation.

## Hardware requirements
We recommend using a machine with at least 8GB of RAM.

## Build the CLI

After cloning the repository, build it, and copy the binary to the root
folder of the repository with the commands:
```sh
cargo build --release 
cp ./target/release/srs_utils ./
```

## Participate

1. For requesting a participation slot, open a GitHub issue in this
   repository. There is a `Request to Participate in SRS Ceremony` template
   for it. You will be assigned a participation number `N` and will be notified
   when your turn arrives.

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
> If a participant does not complete the process within 24 hours,
> their submission will not be accepted and the next participant will be 
> invited to join.
> This is for the sake of liveness, we DO NOT intend to censor anyone.
> (Note that running the update script itself only takes a few minutes on a
> commodity laptop.)

Depending on the prerequisites of the participant, setting up a GitHub
account and and a Rust installation may take some time as well.

### Upload the SRS to our server

You may use our server to upload your updated SRS via SFTP. Simply run:

```sh
sh sftp -v <YOUR_GITHUB_USERNAME>@srs.stg.midnight.tools
put <PATH-TO-UPDATED-SRS> .
```

## Optional: Verify the latest SRS

The latest SRS can be found at [LatestSRS], its size is `3.1GB`. Refer to the
the `PARTICIPANTS.md` file for its SHA256 digest.

After downloading the latest SRS, you may verify that it is structurally
correct with:
```sh
./srs_utils <PATH-TO-DOWNLOADED-SRS> verify-structure
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

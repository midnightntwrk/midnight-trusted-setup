# Trusted Setup for PLONK over BLS12-381  

This is the official repository for the trusted-setup ceremony of the 
[Midnight Network](https://midnight.network/).

The outcome of this ceremony will be a *powers-of-tau* structured reference
string (SRS) over BLS12-381.

> [!IMPORTANT]  
> This ceremony is currently ongoing and you can be a participant! See the
> [instructions](#participate) below.

We build on top of the 
[Filecoin ceremony](https://trusted-setup.filecoin.io/phase1/), an existing
powers-of-tau SRS over BLS12-381 used on [Filecoin](https://filecoin.io/) and
trusted by the Web3 community.

Such initial SRS will be re-randomized several times by various participants.

The chain of update proofs will be stored in the `proofs/` directory of this
repository.

## Build the CLI

After cloning the repository, build it, and copy the binary to the root folder:
```sh
cargo build --release
cp ./target/release/srs_utils ./
```  

## Verify the latest SRS

The latest SRS can be found at [LatestSRS], its size is `3.1GB`. You can find the SHA256sum
of all contributions in `PARTICIPANTS.md`.

After downloading it, you may verify that it is structurally correct with
`./srs_utils <SRS_PATH> verify-structure`.
You may also verify the chain of Schnorr proofs that link it to the
Filecoin SRS with `./srs_utils <SRS_PATH> verify-chain`.
This chain starts at the Filecoin SRS [G1 point](../../blob/main/filecoin_srs_g1_point).
See our [wiki](wiki.md) for details on how to verify the validity of this point.

## Participate

Participants must have a GitHub account and an SSH key linked to it.

1. Open an issue requesting a slot. You will be assigned a participation
number `N` and will be notified when your turn arrives.
2. On your turn, download the [LatestSRS]. You can optionally verify its
structure and `sha256sum` as explained [above](#verify-the-latest-srs).
3. Re-randomize it with `./srs_utils <PATH_TO_LATEST_SRS> update`. This process
will create 2 files: a file named `srs<N>` in the same location where you stored the downloaded SRS and
a file named `proof<N>` in the `proofs/` directory.
4. Compute the SHA digest with `sha256sum srs<USER_NR>`, and add 
a new row to `PARTICIPANTS.md` with your name, GitHub handle, (optional) affiliation, and resulting hash.
5. Open a pull request with only the `proof<N>` file (with a signed commit).
6. Upload the `srs<N>` file to the internet (we provide a server for this purpose if
it helps you, see the [instructions](#upload-the-srs-to-our-server) below).
7. After we verify that your SRS is structurally correct and your Schnorr proof
correctly extends the chain of proofs, your PR will be merged and the next
participant will take over.

> [!WARNING]  
> Participants will be given 24 hours to complete their update process.
> After this time, their turn may be skipped.
>
> This is for the sake of liveness, we DO NOT intend to censor anyone.
> (Note that the update process only takes a few minutes on a commodity laptop.)

### Upload the SRS to our server

You may use our server to upload your SRS via SFTP.
Simply run:

```sh
sftp -v <YOUR_GITHUB_USERNAME>@srs.stg.midnight.tools
put PATH_TO_YOUR_SRS .
```

[LatestSRS]: https://srs.midnight.network/current_srs/challenge19_2p25

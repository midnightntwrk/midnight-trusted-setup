# Midnight's trusted setup ceremony

Welcome to the wiki of [Midnight](https://midnight.network/)'s trusted-setup
ceremony.

The outcome of this ceremony will be a so-called *powers-of-tau* structured
reference string (SRS) over the BLS12-381 elliptic curve of length $2^{25}$
for the [KZG](https://www.iacr.org/archive/asiacrypt2010/6477178/6477178.pdf)
polynomial commitment scheme.

We build upon the last output file `challenge19` of Filecoin's 
`perpetualpowersoftau` ceremony (for the Groth16 proving system) over BLS12-381
described [here](https://github.com/arielgabizon/perpetualpowersoftau). The 
`challenge19` file is hosted 
[via IPFS](https://trusted-setup.filecoin.io/phase1/).

An SRS for Groth16 naturally contains the relevant points of an SRS for
KZG-based PLONK. In particular, we aim for a KZG SRS size of $2^{25}$.
For this purpose, we extracted:

* the first $2^{25}$ points $[1]_1, [\tau]_1,\ldots, [\tau^{2^{25}-1}]_1$
  in $\mathbb{G}_1$, and
* the first $2$ points $[1]_2, [\tau]_2$ in $\mathbb{G}_2$

from the `challenge19` file and verified their structural integrity. 
We discarded the rest of the `challenge19` file (as it is not needed for a 
PLONK SRS). These $2^{25}+2$ points constitute the initial SRS of this
ceremony, and will be re-randomized several times by various participants.

See
[Verifying the extraction from Filecoin's SRS](#verifying-the-extraction-from-filecoins-srs)
for further verification instructions.

## Accessing Midnight's file server

The Midnight team hosts a file server for retrieving and uploading updated
SRS files.

### Downloading the latest SRS

There a public link to the
[latest SRS](https://srs.midnight.network/current_srs/powers_of_tau) at each 
stage of the ceremony. After the ceremony is completed, the same link
will reference the final SRS for long-term retrieval.

### Uploading an updated SRS

Participants may choose their desired channel for communicating their updated
SRS file to the Midnight coordination team.

If a participant wishes to use our file server, they need to request so 
at registration time. The updated SRS file may then be uploaded with:

```sh
sftp -v <YOUR_GITHUB_USERNAME>@sftp.midnight.network
put <PATH-TO-UPDATED-SRS> .
```

## Update process

For collecting randomness, each participant will be prompted to input random
text with their keyboard. This random input is mixed (via the `Blake2b512` hash
function) with randomness collected from the underlying OS. The resulting hash
value is used for seeding the `ChaCha20` RNG from which a `BLS12-381` scalar
is derived. This scalar is the participant's randomness contribution to the 
final SRS.

After every update, the new SRS will be linked to the previous one via a
[Schnorr proof](https://en.wikipedia.org/wiki/Proof_of_knowledge#Schnorr_protocol),
which guarantees that the participant built on top of the previous iteration
and did not start a structurally correct SRS from scratch.

## SRS Structure

Each SRS has a canonical structure, containing:

* the points $[1]_1,[\tau]_1,\ldots,[\tau^{2^{25}-1}]_1$ in $\mathbb{G}_1$, and
* the points $[1]_2,[\tau]_2$ in $\mathbb{G}_2$.

Here, $[\tau^i]_k$ denotes $\tau^i G_k$, where $G_k$ is the designated
generator of $\mathbb{G}_k$, for $k = 1, 2$.

## Verifying the Extraction from Filecoin's SRS
For verifying the chain of updates on top of the SRS from the 
[Filecoin ceremony](https://trusted-setup.filecoin.io/phase1/), we store the 
first $\mathbb{G}_1$ point $[\tau]_1$ from Filecoin's SRS in the file
`filecoin_srs_g1_point`.

Participants and verifiers are encouraged to extract this point themselves.
Our script provides functionality for that:

0. If you have not already, build and copy the binary with
   ```sh
   cargo build --release 
   cp ./target/release/srs_utils ./
   ```

1. Download the 
   [phase1radix2m19](https://trusted-setup.filecoin.io/phase1/phase1radix2m19)
   file.

2. Extract the first $\mathbb{G}_1$ point with
   ```sh
   ./srs_utils <PATH-TO-PHASE1RADIX2M19-file> extract-filecoin-g1-point
   ```
   This will create (possibly overwrite) the file `filecoin_srs_g1_point` in
   the root folder of this repository.

The SHA256 digest of the file `filecoin_srs_g1_point` is expected to be:
```
2d3c62eec11a4e83edd35ca1933a608c0148a10224b674834c64181571c9df21
```
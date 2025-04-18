Welcome to the wiki for the trusted Setup of PLONK over BLS12-381!

We build upon the last output file `challenge19` of Filecoin's `perpetualpowersoftau` ceremony (for the Groth16 proving system) over BLS12-381 described [here](https://github.com/arielgabizon/perpetualpowersoftau). The `challenge19` file is hosted [via IPFS](https://trusted-setup.filecoin.io/phase1/).

An SRS for Groth16 naturally contains the relevant points of an SRS for PLONK. In particular, we work with a PLONK SRS supporting polynomials up to degree $2^{25}-1$. For this purpose, we extracted

* the first $2^{25}$ points $[1]_1, [\tau]_1,\ldots, [\tau^{2^{25}-1}]_1$ in $G_1$, and 
* the first $2$ points $[1]_2, [\tau]_2$ in $G_2$

from the `challenge19` file and verified their structural integrity. We discarded the rest of the `challenge19` file (as it is not needed for a PLONK SRS). These $2^{25}+2$ points constitute the initial SRS for Midnight's setup ceremony.

Please see [Verifying the extraction from Filecoin's `challenge19`](#verifying-the-extraction-from-filecoins-srs) for further verification instructions.

# Accessing the file server
The Midnight team hosts a file server for retrieving and uploading updated SRS files. Furthermore, after the end of Midnight's setup ceremony, the server stores the most recent update of the SRS for long-term retrieval.

There is one public link to the [latest-SRS].

# Instructions on the CLI tool

The following guidelines show how to retrieve, build and use the CLI script under Linux/Bash.

## Building the CLI tool from source
Clone the repository, build the script, and copy the binary:

```

git clone https://github.com/midnightntwrk/trusted_setup && cd trusted_setup

cargo build --release && cp ./target/release/srs_utils ./

```  

## Using the CLI tool
### Updating an existing SRS

Each participant of the ceremony adds his/her own randomness to the final SRS. From the root folder of the repo, run:

```

./srs_utils update <path-to-most-recent-SRS>

```

### Details on the update process
After updating an existing SRS, two files have been created:

* The new/updated SRS `srs<idx>` in the same directory as the old SRS. The new index `idx` is derived from the number of previous update proofs in the `/proofs` folder. For ease of use, we use a canonical naming of SRS files and updates proofs. The old SRS can bear whatever name the user chooses. However, the resulting new SRS `srs<idx>` will automatically follow the canonical naming scheme.

* The corresponding update proof `proof<idx>` in the `/proofs` folder. The update proof will bear the same index as the new SRS file

For collecting randomness, each participant will be prompted to input random text with your keyboard. This random input is mixed (via the `Blake2b512` hash function) with randomness collected from the underlying OS. The resulting hash value is used for seeding the `ChaCha20` RNG, whereby the seeded RNG aids in selecting a random scalar in the scalar field of `BLS12-381`. This random scalar is a participant's contribution of randomness to the final SRS.

### Verifying the structure of an SRS

```

./srs_utils verify-structure <path-to-SRS>

```

### Details on the structure of an SRS
The SRS has a canonical structure:

* the generator in $G_1$ is multiplied by $2^{25}$ consecutive powers $1,\tau^1,\ldots,\tau^{2^{25}-1}$ of a scalar, and
* the generator in $G_2$ is multiplied by $2$ consecutive powers $1,\tau$ of the same scalar

These $2^{25}+2$ points on the curve BLS12-381 constitute an SRS for PLONK. Scalars are taken from the scalar field of BLS12-381.

### Verifying the most recent SRS and the corresponding chain of update proofs

```

./srs_utils verify-chain <path-to-most-recent-SRS>

```

### Details on the structure of an SRS
# Creating a Pull Request on GitHub
To attest to a correct update of the SRS, each participant creates a PR containing:

* The update proof of the corresponding update
* The SHA-256 digest of the updated SRS file

The update proof will be automatically created as part of the calling the `update` command of the CLI tool.

The user only needs to determine the SHA-256 digest. Under Linux/Bash, this can be done via:

```

sha512sum <path-to-SRS>

```

# Verifying the Extraction from Filecoin's `challenge19`

For verifying the chain of updates on top of this initial SRS, we store the first (scaled) $G_1$ point $[\tau]_1$ from Filecoin`s `challenge19` in the file `filecoin_srs_g1_point`.

For verification purposes, participants and verifiers of Midnight's setup ceremony are encouraged extract the first $G_1$ point themselves with the provided CLI tool:

```

./srs_utils <path-to-`phase1radix2m19`-file> extract-filecoin-g1-point

[latest-SRS]: https://srs.midnight
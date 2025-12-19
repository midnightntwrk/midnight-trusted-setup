# TEE contribution (proof31)

This directory contains the attestation and the policy for Trusted Setup contribution #31 (proof31), generated inside a Trusted Execution Environment (TEE) using AMD SEV-SNP (Azure confidential VM). 

## Objectives

- **Verifiability:** Ensure that anyone can independently verify the hashes of the binaries and the TEE's contribution output using the provided attestation and policy.
- **Reproducibility:** Allow third parties to fully rebuild the environment and binaries from source, and verify that the output hashes match those used in the attested contribution.

> ⚠️ All reproducibility guarantees assume the verifier runs the build inside the provided Docker image, which pins the compiler, linker, and all dependencies.

## Contents

- Attestation (JWT): The cryptographically signed statement about the VM and the run, including claims that bind the execution environment and the resulting output/binary hashes.

- Policy (policy.json): the expected VM configuration and expected hashes used to validate the attestation (such as SEV-SNP properties, launch measurement, secure boot, and the pinned hashes).

- Docker image (reproducible build environment): A pinned Docker image used to rebuild the ceremony binaries deterministically.

## Verification

### Build the service

To ensure reproducibility of the ceremony, the entire build process is encapsulated within a simple Docker image. All you need to do is build the Docker image using the following commands:

```bash
git clone https://github.com/midnightntwrk/midnight-trusted-setup.git && cd midnight-trusted-setup/tee_contribution
docker build -f ceremony.Dockerfile  --no-cache -t trusted_setup:0.0.1 .
```

The binaries will be built inside the Docker image, and the hash of each binary will be calculated. Running the built Docker image displays these hashes:

```bash
$ docker run --rm trusted_setup:0.0.1
8f67e82d3f114715f6df8e7462427ff2b6363c997971eaa695e54f22344f2f0f  /artifacts/srs-srv
d7a43a3fdadc84b7fc4172fc825f813a538f3e5d5c3a7a24fb0a01ce5812bf4e  /artifacts/srs_utils
```

You can compare these values with those provided in our attestation and policy.

### Verifying the Ceremony

You can validate the attestation returned by the ceremony server using the `AttestationClient`. To do so, compile the verifier locally and provide it with the attestation from our TEE ceremony setup and the expected policy. 

You can retrieve our TEE ceremony contribution’s attestation output hashes and the artifacts themselves here: [`https://40-76-98-57.sslip.io/calculate`](https://40-76-98-57.sslip.io/calculate)

## Prerequisites

Make sure the following packages are installed:

```bash
sudo apt-get install build-essential libcurl4-openssl-dev libjsoncpp-dev libboost-dev libboost-system1.74.0 cmake nlohmann-json3-dev libssl-dev zlib1g-dev
```

You also need the Azure attestation client:

```bash

wget https://packages.microsoft.com/repos/azurecore/pool/main/a/azguestattestation1/azguestattestation1_1.1.0_amd64.deb
sudo dpkg -i azguestattestation1_1.1.0_amd64.deb
```

### Clone and Compile the AttestationClient

```bash
# The attestation verifier is on a different repository. Please clone this in the same folder where you cloned the Midnight Trusted Setup.
git clone https://github.com/input-output-hk/trusted-setup-management-server.git trusted-setup-server-tee
cd trusted-setup-server-tee/attestation_verifier
cmake . && make
```

### Validate the attestation

The command line to validate the attestation is straightforward. All you need to do is provide the attestation you got from the server, and the policy you created:

```bash
./AttestationClient -v ../../midnight-trusted-setup/tee_contribution/attestation.txt -p ../../midnight-trusted-setup/tee_contribution/policy.json
```

If everything matches, you’ll see output confirming that the attestation is valid and the VM state complies with your policy:

```bash
JWT token read from ../../midnight-trusted-setup/tee_contribution/attestation.txt
Verifying JWT token:
...
Cert URL header: https://sharedeus2.eus2.attest.azure.net/certs
Key ID header: J0pAPdfXXHqWWimgrH853wMIdh5/fLe1z6uSXYPXCa0=
Using platform leaf kid=J0pAPdfXXHqWWimgrH853wMIdh5/fLe1z6uSXYPXCa0= for JWT verification
Warning: Lifetime verification has been disabled
Verifying the JWT...
JWT token is valid.
Policy check passed for attestation-type: "sevsnpvm"
Policy check passed for compliance-status: "azure-compliant-cvm"
Policy check passed for vm_id: "ABD9EA99-D837-4F68-AD3E-C87C88006AFA"
Policy check passed for secureboot: true
Policy check passed for kerneldebug-enabled: false
Policy check passed for imageId: "02000000000000000000000000000000"
Policy check passed for microcode-svn: 219
Policy check passed for snpfw-svn: 24
Policy check passed for launch_measurement: "6a063be9dd79f6371c842e480f8dc3b5c725961344e57130e88c5adf49e8f7f6c79b75a5eb77fc769959f4aeb2f9401e"
Policy check passed for srs_hash: 35d66812707d05d25509331ef61c7e5ef754fbfb71d95010bd68d4c81a3775a4
Policy check passed for proof_hash: 035710d5a41f36c595a7c4f6bd22a439bd522589ef6766d87db992cd869d29e6
Policy check passed for srs_utils_hash: d7a43a3fdadc84b7fc4172fc825f813a538f3e5d5c3a7a24fb0a01ce5812bf4e
Policy check passed for srs_srv_hash: 8f67e82d3f114715f6df8e7462427ff2b6363c997971eaa695e54f22344f2f0f
Attestation compliant with the policy!
Policy check passed.
Attestation verified successfully.
```

This confirms:

- **The VM was running in a valid SEV-SNP configuration (matching `launch_measurement`, `vm_id`, etc.)**
- **The hashes of the rebuilt artifacts match the hashes verified by the policy and attestation.**

You can as well, if you wish, inspect the attestation yourself using:

- [jwt.io](https://jwt.io/)
- A local tool of your choice (such as Python’s `pyjwt`, Rust’s `jsonwebtoken`...)

The attestation contains cryptographic claims about the VM, including launch measurements, secure boot settings, and isolation parameters. It is signed by Microsoft.
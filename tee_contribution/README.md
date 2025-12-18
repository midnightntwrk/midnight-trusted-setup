# TEE contribution (proof31)

This repository contains the attestation and the policy for Trusted Setup contribution #31 (proof31), generated inside a Trusted Execution Environment (TEE) using AMD SEV-SNP (Azure confidential VM).

## Contents

Attestation (JWT): a cryptographically signed statement about the VM and the run, including claims that bind the execution environment and the resulting output/binary hashes.

Policy (policy.json): the expected VM configuration and expected hashes used to validate the attestation (e.g. SEV-SNP properties, launch measurement, secure boot, and the pinned hashes).

## Verification

To validate the attestation against the policy (and to reproduce/rebuild the binaries if you want), follow the step-by-step guide here:

Notion documentation: https://futuristic-stoplight-f0f.notion.site/Midnight-Trusted-Setup-Reproducible-TEE-Deployment-Documentation-224b5d65750580ca95a6e4c7e7b50b73
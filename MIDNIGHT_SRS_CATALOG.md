# Midnight SRS Catalog

This document lists the available extended SRS files containing both
coefficient and Lagrange representations for various sizes.

Each extended SRS file includes:
- `4` bytes representing the length `n` (in little-endian),
- `n` G1 points in coefficient (monomial) basis: `[1, τ, τ², ..., τⁿ⁻¹]₁`,
- `n` G1 points in Lagrange basis: `[L₀(τ), L₁(τ), ..., Lₙ₋₁(τ)]₁`,
- `2` G2 points: `[1, τ]₂`.

## Available Files

| Size (k) | File Size (bytes) | SHA256 Checksum                                                    |
|----------|-------------------|--------------------------------------------------------------------|
|        1 |               772 | `bbe04fe3c70d0c138447cb086b4baddc30cb8bb2a004114bc02e6f739516280e` |
|        2 |             1,156 | `80e15568fa1a0117db893239be7fa5e34a6bcc3a8c3bfa7709534b9cb88eb6c1` |
|        3 |             1,924 | `4be827a6472193df80d8f08b4b25a85baef436fdd1965d89b6af89f4ec4e99e2` |
|        4 |             3,460 | `232f401fad10c7ddf8828d2aa4c85c6506c5da09795998cecaeb9f75fc8f6ada` |
|        5 |             6,532 | `0a1c9229f315fc1868ff25f668fb83aec4d09f4f23a706b5197c692c619d72c6` |
|        6 |            12,676 | `cf2ad6be7d0fedf5bec2aaa35f6be4aca33053d74268fdf5aa54fcb2891ea6df` |
|        7 |            24,964 | `e82ae890c080188355f37feaffe91372584cd810615082d9143d4dec0453fd9d` |
|        8 |            49,540 | `909b707551eaaea79828e883cde6fc46ab15986c3b1d791bed462c9e2805c933` |
|        9 |            98,692 | `b9009f1098bcefffec3c461ab3a5e3a17f7e5599f0f08c70fcdc55a89227bcbd` |
|       10 |           196,996 | `46b2290933cbed4c378889e4ba971f1a92888331ffb09466acd4ff61a1e2cb42` |
|       11 |           393,604 | `9901589d7956ff58be0d85569b2f455b77b58c3758026ffb5bbe4807000b96d1` |
|       12 |           786,820 | `ef08eb3fcf62df8f72c515cffa027e681808b530cb016eea104115545ef6d5c8` |
|       13 |         1,573,252 | `d3324910969c4cc54143b8045b649e5c3a4bd5fb7b8f85fe1b770f640ce1c803` |
|       14 |         3,146,116 | `fc253016885ec830e97808c9ec920bb5cab5c21af590380a6cb5eb0538e2b244` |
|       15 |         6,291,844 | `724c7c3d779148bb113c7ee9c034b2f27db16e6bdf315fde90105a9bad00b1de` |
|       16 |        12,583,300 | `09c877216d6589b370263e18af40a030a901b41a7a7c37ef58c9901db41f05c6` |
|       17 |        25,166,212 | `4a9ef6c7c0619aab74eede44b13e753e3ba54508a02dd3b7106a949aabb73b74` |
|       18 |        50,332,036 | `e8436dc5d8b598f169c127c745135d889744007e6d384ff126df8d1332522f86` |
|       19 |       100,663,684 | `8e8dc15c4362f05c912f1e770559a3945db3e58a374def416ed5d3e65ad5b10e` |
|       20 |       201,326,980 | `1cc62978558fdc1e445cd70cfd9a86ec3c2e2151b6d74811232d37faf9133ff1` |
|       21 |       402,653,572 | `9cf1644a87f0f027ae5fc6278f91d823a6334ff3e338a29e2f2ef57d071ed64d` |
|       22 |       805,306,756 | `e8ad5eed936d657a0fb59d2a55ba19f81a3083bb3554ef88f464f5377e9b2c2f` |
|       23 |     1,610,613,124 | `09399d05f9f50875dfdd87dc9903d40c897eaafa9ec8cbb08bace853ecc36c0c` |
|       24 |     3,221,225,860 | `b0e6fa7a4ab4a79a1e6560966f267556409db44bab6d5fab3711ad6c6b623207` |
|       25 |     6,442,451,332 | `3289a751c938988cd2f54154d8722d1eda2cd11593064afdde82099b24ff4a58` |

**Note:** File size formula: `4 + 2 × n × 96 + 2 × 192` bytes, where `n = 2^k`.

## Download

The extended SRS files can be downloaded from:

```
http://midnight-s3-fileshare-dev-eu-west-1.s3.eu-west-1.amazonaws.com/bls_midnight_2p<k>
```

Replace `<k>` with the desired size parameter (from 1 to 25).

## Verification

Verify the integrity of a downloaded extended SRS file:

```bash
sha256sum <PATH-TO-EXTENDED-SRS>
```

Verify consistency with respect to the official
[powers-of-tau](https://srs.midnight.network/current_srs/powers_of_tau):

```bash
cargo run --release --bin srs_consistency <PATH-TO-POWERS-OF-TAU> <PATH-TO-EXTENDED-SRS>
```
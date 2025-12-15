#!/usr/bin/env bash

# Should be run as root or with sudo
set -euo pipefail

# Remove existing sources.list
# Necessary to only target the pinned snapshot and avoid any other sources.
rm -f /etc/apt/sources.list

# This script sets up a reproducible Ubuntu environment using a specific snapshot.
# Use the same than the trusted setup docker image.
tee /etc/apt/sources.list >/dev/null <<'EOF'
deb [arch=amd64] https://snapshot.ubuntu.com/ubuntu/20251013T000000Z jammy main restricted universe
deb [arch=amd64] https://snapshot.ubuntu.com/ubuntu/20251013T000000Z jammy-updates main restricted universe
deb [arch=amd64] https://snapshot.ubuntu.com/ubuntu/20251013T000000Z jammy-security main restricted universe
EOF

# Update package lists and clean up
apt-get clean
apt-get update

# Remove daily apt services and timers to prevent automatic updates
systemctl disable --now apt-daily.service apt-daily.timer
systemctl disable --now apt-daily-upgrade.service apt-daily-upgrade.timer

# Disable unattented updates
apt-get purge -y unattended-upgrades

# Install essential build tools & cache them
apt-cache madison build-essential libcurl4-openssl-dev libjsoncpp-dev libboost-dev libboost-system1.74.0 cmake nlohmann-json3-dev libssl-dev zlib1g-dev
apt-get install -y --no-install-recommends \
  zlib1g-dev=1:1.2.11.dfsg-2ubuntu9.2 \
  libssl3=3.0.2-0ubuntu1.20 \
  libssl-dev=3.0.2-0ubuntu1.20 \
  build-essential=12.9ubuntu3 \
  libcurl4-openssl-dev=7.81.0-1ubuntu1.21 \
  libjsoncpp-dev=1.9.5-3 \
  libboost-dev=1.74.0.3ubuntu7 \
  libboost-system1.74.0 \
  cmake=3.22.1-1ubuntu1.22.04.2 \
  nlohmann-json3-dev=3.10.5-2 \
  git \
  curl \
  ca-certificates \
  wget

# Hold essential packages to prevent them from being updated
# This is important for reproducibility and to avoid breaking the environment
PKGS=$(dpkg -l \
  | awk '/^ii  (build-essential|gcc|g\+\+|cpp|make|dpkg-dev|cmake|libboost.*|libcurl4-openssl-dev|libjsoncpp-dev|nlohmann-json3-dev|zlib1g-dev|libssl|libc|libstdc\+\+|libgcc|lib.*san)/{print $2}')
apt-mark hold $PKGS

# Show held packages for verification
apt-mark showhold

# Azure Attestation SDK needs to be installed here too, preferably the v1.1.0.
wget https://packages.microsoft.com/repos/azurecore/pool/main/a/azguestattestation1/azguestattestation1_1.1.0_amd64.deb
dpkg -i azguestattestation1_1.1.0_amd64.deb
rm azguestattestation1_1.1.0_amd64.deb

# Docker
# The docker image builds all deterministic binaries and exports them under /artifacts.
# We build the image inside the VM (TEE), then extract /artifacts to /root/tee.
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
chmod a+r /etc/apt/keyrings/docker.asc

# Set up the stable repository
tee /etc/apt/sources.list.d/docker.sources <<EOF
Types: deb
URIs: https://download.docker.com/linux/ubuntu
Suites: $(. /etc/os-release && echo "${UBUNTU_CODENAME:-$VERSION_CODENAME}")
Components: stable
Signed-By: /etc/apt/keyrings/docker.asc
EOF

# Update and install docker packages
apt-get update
apt-get install -y --no-install-recommends \
  docker-ce \
  docker-ce-cli \
  containerd.io \
  docker-buildx-plugin \
  docker-compose-plugin
systemctl start docker

# Build the docker image
# NOTE: toolchains, snapshots, and commits are defined in the Dockerfile.
docker build -f ceremony.Dockerfile --no-cache -t trusted-setup-artifacts:latest .

# Extract /artifacts to /root/tee
mkdir -p /root/tee
CID="$(docker create trusted-setup-artifacts:latest)"
docker cp "${CID}:/artifacts/." /root/tee/
docker rm "${CID}"

# Print artifact hashes
# Allows to compare the hashes calculated here with the one in the attestation.
# It is also possible to copy the binairies from the /artifacts directory and run the
# hash calculation locally.
if [[ -f /root/tee/hashes.txt ]]; then
  echo "Artifacts hashes:"
  cat /root/tee/hashes.txt
fi

# Caddy
# The srs_server only runs in HTTP. We terminate TLS with Caddy and proxy to localhost:8080.
# We don't have a ready domain, so we use a zero-setup host like <ip-with-dashes>.sslip.io
apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | tee /etc/apt/sources.list.d/caddy-stable.list
chmod o+r /usr/share/keyrings/caddy-stable-archive-keyring.gpg
chmod o+r /etc/apt/sources.list.d/caddy-stable.list
apt update
apt-get install -y --no-install-recommends caddy

PUBLIC_IP="${PUBLIC_IP:-$(curl -fsS https://api.ipify.org || true)}"
if [[ -z "${PUBLIC_IP}" ]]; then
  echo "ERROR: Could not determine public IP. Set PUBLIC_IP env var and rerun."
  exit 1
fi
CADDY_HOST="${CADDY_HOST:-$(echo "${PUBLIC_IP}" | tr '.' '-')}.sslip.io"

cat >/etc/caddy/Caddyfile <<EOF
${CADDY_HOST} {
    encode zstd gzip
    reverse_proxy 127.0.0.1:8080
}
EOF

systemctl enable --now caddy
caddy fmt --overwrite /etc/caddy/Caddyfile
systemctl reload caddy

echo "Caddy configured. HTTPS endpoint: https://${CADDY_HOST}/"

# Clone project repositories
# We only clone what is needed for the AttestationClient build.
rm -rf tee
git clone https://github.com/input-output-hk/trusted-setup-management-server.git tee
cd tee && git checkout 1b374bd8bda535999515f4c80c5fa3ec1c66453c

# Copy the test hashes to the tee directory
cp test_hashes.bin /root/tee/test_hashes.bin

# Build the attestation client
cd attestation_verifier
cmake .
make
sha256sum AttestationClient

# Copy the AttestationClient binary to the tee directory
cp AttestationClient /root/tee/AttestationClient
cd ../..

# Copy the extra assets to the tee directory
cp -r ../proofs /root/tee/
rm -rf tee

# Make binaries executable
chmod +x /root/tee/srs-srv
chmod +x /root/tee/AttestationClient
chmod +x /root/tee/srs_utils

# Install the srs server service
cp /root/tee/srs_server.service /etc/systemd/system/srs_server.service

# Install and configure srs server service
systemctl daemon-reload
systemctl enable srs_server.service

# Disabling entropy, remove machine ID & logs

# This won't affect the randomness of the VM
# as getrandom, when used in an CVM image
# relies on the processor instruction
# RDRAND, getting the seed while booting, without saving it on the
# disk, preserving the reproducibility and a strong entropy.
rm -f /var/lib/systemd/random-seed
systemctl disable --now systemd-random-seed.service
systemctl mask systemd-random-seed.service

# Clean up apt cache
apt-get clean
rm -rf /var/lib/apt/lists/* /var/cache/apt/archives
rm -rf /var/lib/dpkg/info/*.list

# Disable SSH permanently
if [[ "${DEPROVISION:-0}" == "1" ]]; then
  rm -rf ~/.ssh
  rm -f /etc/ssh/ssh_host_*
  systemctl disable ssh
  systemctl mask ssh
  echo "SSH has been disabled permanently."
else
  echo "SSH will not be disabled (testing mode)"
fi

# Delete script after execution
# This is to ensure that the script does not remain on the system after execution.
SCRIPT_PATH=$(realpath "$0")
rm -f "$SCRIPT_PATH"

# Cleanup
set -e
truncate -s0 ~/.bash_history || true
rm -rf ./.cache || true
rm -rf ./.local/share/nano || true

cd /root

# Final cleanup
journalctl --rotate || true
journalctl --vacuum-time=1s || true
find /var/log -type f -exec truncate -s0 {} + || true

if [[ "${DEPROVISION:-0}" == "1" ]]; then
  echo "Deprovisioning VM (removing user, SSH keys, history)..."
  waagent -deprovision+user -force
  sleep 5
  shutdown -h now
else
  echo "Skipping deprovision (testing mode)"
fi
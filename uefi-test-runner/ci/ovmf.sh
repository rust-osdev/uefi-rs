#!/bin/bash
# Download OVMF from kraxel's repo

set -e

wget https://www.kraxel.org/repos/jenkins/edk2/edk2.git-ovmf-x64-0-20180807.267.g447b08b3d2.noarch.rpm -O ovmf.rpm

# Need to use bsdtar to extract RPMs
bsdtar xvf ovmf.rpm

# Extract the right files
cp 'usr/share/edk2.git/ovmf-x64/OVMF_CODE-pure-efi.fd' 'uefi-test-runner/OVMF_CODE.fd'
cp 'usr/share/edk2.git/ovmf-x64/OVMF_VARS-pure-efi.fd' 'uefi-test-runner/OVMF_VARS.fd'

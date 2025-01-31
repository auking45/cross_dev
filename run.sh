#!/bin/bash

set -euo pipefail

#===========================================================
# Constants
#===========================================================
SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
SCRIPT_DIR="$(dirname "${SCRIPT_PATH}")"

ROOT_DIR=$(git rev-parse --show-toplevel)
WORK_DIR="${ROOT_DIR}/.work"
COMMON_DIR="${WORK_DIR}/common"
TOOLCHAIN_DIR="${COMMON_DIR}/toolchains"
RISCV64_TOOLCHAIN_PATH="${TOOLCHAIN_DIR}/riscv64/bin"

#===========================================================
# Functions
#===========================================================
function help {
    echo "$0 <task> <args>"
    echo "Tasks:"
    compgen -A function | cat -n
}

function install_dependencies {
    echo "🚀 Installing dependencies..."

    # Install dependencies
    sudo apt-get update
    sudo apt-get install -y \
        build-essential \
        cmake \
        git \
        ninja-build

    echo "🎉 Dependencies installed!"
}

function prepare_toolchains {
    echo "🚀 Preparing toolchains..."

    UBUNTU_VERSION=$(lsb_release -rs)
    TAG="2025.01.20"
    TOOLCHAIN_URL="https://github.com/riscv-collab/riscv-gnu-toolchain/releases/download"
    FILENAME="riscv64-glibc-ubuntu-${UBUNTU_VERSION}-llvm-nightly-${TAG}-nightly.tar.xz"

    if [ ! -d "${RISCV64_TOOLCHAIN_PATH}" ]; then
        mkdir -p "${TOOLCHAIN_DIR}"
        cd "${TOOLCHAIN_DIR}"
        wget "${TOOLCHAIN_URL}/${TAG}/${FILENAME}"
        tar -xf "${FILENAME}"
        rm "${FILENAME}"
        cd -
    fi

    echo "🎉 Toolchains prepared!"
}

function setup {
    echo "🚀 Setting up workspace..."

    mkdir -p "${WORK_DIR}"
    mkdir -p "${COMMON_DIR}"

    prepare_toolchains

    echo "🎉 Workspace setup complete!"
}

#===========================================================
# Script begins here
#===========================================================
TIMEFORMAT="Task completed in %3lR"
time ${@:-help}

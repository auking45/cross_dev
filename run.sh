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
RISCV64_TOOLCHAIN_PATH="${TOOLCHAIN_DIR}/riscv/bin"
RISCV64_CROSS_TOOLCHAIN="${RISCV64_TOOLCHAIN_PATH}/riscv64-unknown-linux-gnu-"

QEMU_VERSION="9.2.0"
QEMU_DIR="${COMMON_DIR}/qemu-${QEMU_VERSION}"
QEMU_BUILD_DIR="${QEMU_DIR}/build"
QEMU_BIN="${QEMU_BUILD_DIR}/qemu-system-riscv64"

RISCV_DIR="${WORK_DIR}/riscv"
RISCV_IMAGES_DIR="${RISCV_DIR}/images"

OPENSBI_BIN="fw_jump.bin"

export PATH="${RISCV64_TOOLCHAIN_PATH}:${PATH}"

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
        ninja-build \
        python3-venv

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

function prepare_qemu {
    echo "🚀 Preparing QEMU..."

    if [ ! -d "${QEMU_DIR}" ]; then
        mkdir -p "${QEMU_DIR}"
        cd "${QEMU_DIR}"
        wget "https://download.qemu.org/qemu-${QEMU_VERSION}.tar.xz"
        tar -xf "qemu-${QEMU_VERSION}.tar.xz"
        rm "qemu-${QEMU_VERSION}.tar.xz"
        cd -
    fi

    if [ ! -f "${QEMU_BIN}" ]; then
        mkdir -p "${QEMU_BUILD_DIR}"
        cd "${QEMU_BUILD_DIR}"
        "${QEMU_DIR}/qemu-${QEMU_VERSION}/configure" \
            --target-list=riscv64-softmmu \
            --prefix="${QEMU_DIR}"
        make -j$(nproc)
        cd -
    fi

    "${QEMU_BIN}" --version

    echo "🎉 QEMU prepared!"
}

function prepare_opensbi {
    echo "🚀 Preparing OpenSBI..."

    REPO="https://github.com/riscv-software-src/opensbi.git"
    BRANCH="v1.6"
    SRC_DIR="${RISCV_DIR}/opensbi"

    if [ ! -d "${SRC_DIR}" ]; then
        git clone -b "${BRANCH}" "${REPO}" "${SRC_DIR}"
    fi

    cd "${SRC_DIR}"

    export CROSS_COMPILE="${RISCV64_CROSS_TOOLCHAIN}"

    make PLATFORM=generic

    cp -f ./build/platform/generic/firmware/${OPENSBI_BIN} "${RISCV_IMAGES_DIR}/"

    cd -

    echo "🎉 OpenSBI prepared!"
}

function prepare_linux {
    echo "🚀 Preparing Linux..."

    REPO="git://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git"
    BRANCH="v6.13"
    SRC_DIR="${COMMON_DIR}/linux"
    TARGET_ARCH="riscv"

    if [ ! -d "${SRC_DIR}" ]; then
        git clone --depth 1 -b "${BRANCH}" --single-branch "${REPO}" "${SRC_DIR}"
    fi

    cd "${SRC_DIR}"

    export ARCH="${TARGET_ARCH}"
    export CROSS_COMPILE="${RISCV64_CROSS_TOOLCHAIN}"

    make defconfig
    # if there are extra configs
    ./scripts/kconfig/merge_config.sh .config "${SCRIPT_DIR}/configs/linux/extra.config"

    make -j$(nproc)

    cp -f ./arch/${TARGET_ARCH}/boot/Image "${RISCV_IMAGES_DIR}/"

    cd -

    echo "🎉 Linux prepared!"
}

function setup {
    echo "🚀 Setting up workspace..."

    mkdir -p "${WORK_DIR}"
    mkdir -p "${COMMON_DIR}"
    mkdir -p "${RISCV_IMAGES_DIR}"

    prepare_toolchains
    prepare_qemu
    prepare_opensbi
    prepare_linux

    echo "🎉 Workspace setup complete!"
}

#===========================================================
# Script begins here
#===========================================================
TIMEFORMAT="Task completed in %3lR"
time ${@:-help}

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

LINUX_BUILD_DIR="${RISCV_DIR}/linux"

BUILDROOT_DIR="${COMMON_DIR}/buildroot"
BR_ORG_CUSTOM_DIR="${SCRIPT_DIR}/custom_buildroot"
BR_CUSTOM_DIR="${COMMON_DIR}/custom_buildroot"
BR_OVERLAY_DIR="${BR_CUSTOM_DIR}/board/riscv/overlay"

BR_RISCV_DIR="${RISCV_DIR}/buildroot"
BR_RISCV_OUTPUT_DIR="${BR_RISCV_DIR}/output"
BR_RISCV_CONFIG="qemu_riscv64_virt_riscv_defconfig"

SSH_DIR="${BR_OVERLAY_DIR}/root/.ssh"
SSH_KEY="${SSH_DIR}/id_rsa"
SSH_PORT=12345

OPENSBI_BIN="fw_jump.bin"
LINUX_BIN="Image"
ROOTFS_BIN="rootfs.img"


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
        python3-venv \
        cpio

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

    mkdir -p "${LINUX_BUILD_DIR}"

    cd "${LINUX_BUILD_DIR}"

    export ARCH="${TARGET_ARCH}"
    export CROSS_COMPILE="${RISCV64_CROSS_TOOLCHAIN}"

    make O="${LINUX_BUILD_DIR}" -C "${SRC_DIR}" defconfig
    # if there are extra configs
    "${SRC_DIR}/scripts/kconfig/merge_config.sh" .config "${SCRIPT_DIR}/configs/linux/extra.config"

    make -j$(nproc)

    cp -f "${LINUX_BUILD_DIR}/arch/${TARGET_ARCH}/boot/${LINUX_BIN}" "${RISCV_IMAGES_DIR}/"

    cd -

    echo "🎉 Linux prepared!"
}

function build_buildroot {
    cd "${BR_RISCV_OUTPUT_DIR}"

    # Remove any trailing whitespace from PATH which causes buildroot to fail in WSL
    PATH="$(echo $PATH | tr -d ' \t\n')"

    make -j$(nproc)

    cp -f "${BR_RISCV_OUTPUT_DIR}/images/rootfs.ext2" "${RISCV_IMAGES_DIR}/${ROOTFS_BIN}"

    cd -
}

function prepare_buildroot {
    echo "🚀 Preparing Buildroot..."

    REPO="http://github.com/buildroot/buildroot"
    BRANCH="2024.11.1"
    SRC_DIR="${BUILDROOT_DIR}"
    TARGET_ARCH="riscv"

    if [ ! -d "${SRC_DIR}" ]; then
        git clone --depth 1 -b "${BRANCH}" --single-branch "${REPO}" "${SRC_DIR}"
    fi

    cd "${SRC_DIR}"

    # In order not to copy intermediate files into the original overlay directory
    cp -rf "${BR_ORG_CUSTOM_DIR}" "${COMMON_DIR}"

    prepare_ssh_key

    make O="${BR_RISCV_OUTPUT_DIR}" BR2_EXTERNAL="${BR_CUSTOM_DIR}" "${BR_RISCV_CONFIG}"

    build_buildroot

    cd -

    echo "🎉 Buildroot prepared!"
}

function prepare_ssh_key {
    echo "🚀 Preparing SSH..."

    mkdir -p "${SSH_DIR}"

    if [ -f "${SSH_DIR}/id_rsa" ] && [ -f "${SSH_DIR}/authorized_keys" ]; then
        echo "🎉 SSH key has already been created."
        return
    fi

    ssh-keygen -t rsa -N "" -f "${SSH_DIR}/id_rsa" <<< y 2>&1 >/dev/null
    cat "${SSH_DIR}/id_rsa.pub" >> "${SSH_DIR}/authorized_keys"

    echo "🎉 SSH prepared!"
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
    prepare_buildroot

    echo "🎉 Workspace setup complete!"
}


function run_qemu {
    echo "🚀 Running QEMU..."

    ARGS=(
        -machine virt
        -nographic
        -smp 4
        -m 2G
        -serial mon:stdio
        -semihosting-config enable=on
        -bios "${RISCV_IMAGES_DIR}/${OPENSBI_BIN}"
        -kernel "${RISCV_IMAGES_DIR}/${LINUX_BIN}"
        -append "console=ttyS0 ro root=/dev/vda init=/sbin/init"
        -drive file="${RISCV_IMAGES_DIR}/${ROOTFS_BIN}",if=none,format=raw,id=hd0
        -device virtio-blk-device,drive=hd0
        -netdev user,id=net0,hostfwd=tcp::${SSH_PORT}-:22
        -device virtio-net-device,netdev=net0
    )

    "${QEMU_BIN}" "${ARGS[@]}" "${@}"
}

function run_ssh {
    echo "🚀 Connecting via SSH..."

    SSH_COMMAND="ssh -i ${SSH_KEY} root@localhost -p ${SSH_PORT} -o StrictHostKeyChecking=no"
    echo "Executing: ${SSH_COMMAND}"

    ${SSH_COMMAND}
}

#===========================================================
# Script begins here
#===========================================================
TIMEFORMAT="Task completed in %3lR"
time ${@:-help}

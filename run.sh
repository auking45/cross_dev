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

#===========================================================
# Functions
#===========================================================
function help {
    echo "$0 <task> <args>"
    echo "Tasks:"
    compgen -A function | cat -n
}

function setup {
    echo "🚀 Setting up workspace..."

    mkdir -p "${WORK_DIR}"
    mkdir -p "${COMMON_DIR}"

    echo "🎉 Workspace setup complete!"
}

#===========================================================
# Script begins here
#===========================================================
TIMEFORMAT="Task completed in %3lR"
time ${@:-help}

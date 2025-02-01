# GDB init file
#
# This file contains GDB commands to be executed every time GDB is started.
#
# Usage:
#   gdb-multiarch -x .gdbinit --cd=./scripts/gdb
#   gdb-multiarch -x .gdbinit --cd=./scripts/gdb --args ./build/program

source .gdbinit.py

# Use gef-remote to connect to QEMU debug server
# https://github.com/hugsy/gef
gef-remote --qemu-user localhost 1234

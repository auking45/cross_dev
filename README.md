# Cross Development Environment

This repository provides a cross-development environment for RISC-V using QEMU, OpenSBI, Linux, and Buildroot. The setup script automates the preparation of toolchains, QEMU, OpenSBI, Linux kernel, and Buildroot.

## Prerequisites

Ensure you have the following dependencies installed on your system:

- Git
- wget
- tar
- build-essential
- cmake
- ninja-build
- python3-venv
- cpio

You can install these dependencies using the following command:

```bash
sudo apt-get update
sudo apt-get install -y git wget tar build-essential cmake ninja-build python3-venv cpio
```

## Setup

To set up the cross-development environment, run the `run.sh` script with the `setup` task:

```bash
./run.sh setup
```

This will perform the following steps:

1. Install dependencies
2. Prepare toolchains
3. Prepare QEMU
4. Prepare OpenSBI
5. Prepare Linux kernel
6. Prepare Buildroot

## Usage

### Running QEMU

To run QEMU with the prepared images, use the `run_qemu` task:

```bash
./run.sh run_qemu
```

### Connecting via SSH

To connect to the running QEMU instance via SSH, use the `run_ssh` task:

```bash
./run.sh run_ssh
```

### Running GDB

To run GDB with the prepared environment, use the `run_gdb` task:

```bash
./run.sh run_gdb
```

## Directory Structure

- `.work`: Contains the working directories for toolchains, QEMU, OpenSBI, Linux, and Buildroot.
- `scripts`: Contains custom scripts and configurations.
- `images`: Contains the generated images for QEMU.

## Customization

You can customize the setup by modifying the `run.sh` script and the configuration files in the `scripts` directory.

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Acknowledgements

This project uses the following open-source projects:

- [QEMU](https://www.qemu.org/)
- [OpenSBI](https://github.com/riscv/opensbi)
- [Linux Kernel](https://www.kernel.org/)
- [Buildroot](https://buildroot.org/)

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements or bug fixes.

## Contact

For any questions or support, please contact [auking45@gmail.com](mailto:auking45@gmail.com).

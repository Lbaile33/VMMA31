# VMMA31: Virtual Machine for 32-bit Architecture

Welcome to **VMMA31**, a virtual machine designed for a custom 32-bit architecture. This project allows you to run assembly programs using a tailored instruction set, providing a simple environment for experimenting with low-level programming.

---

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [License](#license)

---

## Installation

To set up VMMA31 on your system, follow these steps:

1. **Download the Files**:
   - Clone the repository using Git:
     ```sh
     git clone https://github.com/lbaile33/vmma31.git
     ```

2. **Navigate to the Folder**:
   - Change into the project directory:
     ```sh
     cd vmma31
     ```

3. **Build the Project**:
   - Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.
   - Clean and build the project with these commands:
     ```sh
     cargo clean
     cargo build --release
     ```

---

## Usage

To run a program with VMMA31, follow these steps:

1. **Prepare Your Assembly File**:
   - Create an assembly file (e.g., `my_test_file.v`) with your program written in the VMMA31 instruction set.

2. **Execute the Program**:
   - Run the virtual machine with your file using this command:
     ```sh
     cargo run --release my_test_file.v
     ```

---

## License

This project is licensed under the [MIT License](LICENSE). See the LICENSE file for more details.

---

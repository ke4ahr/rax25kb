================================================================================
Building Documentation
================================================================================

rax25kb - AX.25 KISS Bridge with Multi-Port Cross-Connect Support

:Version: 2.0.0
:Author: Kris Kirby, KE4AHR
:Date: December 2025

================================================================================
Quick Start
================================================================================

For the impatient::

    # Install Rust (if not already installed)
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

    # Clone and build
    git clone https://github.com/ke4ahr/rax25kb.git
    cd rax25kb
    cargo build --release

    # Binary will be at: target/release/rax25kb

================================================================================
Prerequisites
================================================================================

System Requirements
--------------------------------------------------------------------------------

**Operating Systems**

* Linux (primary platform, tested on Ubuntu, Debian, Fedora, Arch)
* macOS (tested on macOS 12+)
* Windows (tested on Windows 10/11)
* BSD systems (should work, not regularly tested)

**Hardware Requirements**

* CPU: Any modern CPU (x86_64, ARM, ARM64, RISC-V)
* RAM: 512 MB minimum, 1 GB recommended
* Disk: 50 MB for binaries and dependencies
* Serial ports: USB-to-serial adapters, native serial ports, or USB TNCs

Software Dependencies
--------------------------------------------------------------------------------

**Required**

* **Rust toolchain** (1.70.0 or later)

  - rustc (compiler)
  - cargo (build system and package manager)
  - rustup (toolchain manager, recommended)

**Optional Build Tools**

* **git** - for cloning the repository
* **pkg-config** - for system library detection (Linux)
* **libudev-dev** - for serial port detection (Linux)

**Runtime Dependencies**

* **libc** - C standard library (provided by OS)
* **libudev** - device manager library (Linux only)

Installing Rust
--------------------------------------------------------------------------------

Official Installation (Recommended)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Linux / macOS / BSD**::

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env

**Windows**

Download and run: https://rustup.rs/

This installs:

* ``rustc`` - The Rust compiler
* ``cargo`` - Package manager and build tool
* ``rustup`` - Toolchain version manager

Verify Installation::

    rustc --version    # Should show: rustc 1.7x.x
    cargo --version    # Should show: cargo 1.7x.x

Package Manager Installation (Alternative)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Ubuntu / Debian**::

    sudo apt update
    sudo apt install cargo rustc

**Fedora / RHEL**::

    sudo dnf install cargo rust

**Arch Linux**::

    sudo pacman -S rust

**macOS (Homebrew)**::

    brew install rust

**Note**: Package manager versions may be older. Use rustup for latest version.

Installing System Dependencies
--------------------------------------------------------------------------------

Linux (Debian/Ubuntu)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

::

    sudo apt update
    sudo apt install \
        build-essential \
        pkg-config \
        libudev-dev \
        git

Linux (Fedora/RHEL)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

::

    sudo dnf install \
        gcc \
        pkg-config \
        systemd-devel \
        git

Linux (Arch)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

::

    sudo pacman -S \
        base-devel \
        systemd \
        git

macOS
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Install Xcode Command Line Tools::

    xcode-select --install

Windows
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

* Install Visual Studio Build Tools or Visual Studio Community
* Ensure "Desktop development with C++" workload is selected
* Rust will use MSVC toolchain automatically

================================================================================
Obtaining the Source Code
================================================================================

Clone from GitHub
--------------------------------------------------------------------------------

**HTTPS (recommended for most users)**::

    git clone https://github.com/ke4ahr/rax25kb.git
    cd rax25kb

**SSH (for contributors with GitHub SSH keys)**::

    git clone git@github.com:ke4ahr/rax25kb.git
    cd rax25kb

Download as Archive
--------------------------------------------------------------------------------

**Release tarball**::

    wget https://github.com/ke4ahr/rax25kb/archive/refs/tags/v2.0.0.tar.gz
    tar xzf v2.0.0.tar.gz
    cd rax25kb-2.0.0

**Release zip**::

    wget https://github.com/ke4ahr/rax25kb/archive/refs/tags/v2.0.0.zip
    unzip v2.0.0.zip
    cd rax25kb-2.0.0

================================================================================
Building the Project
================================================================================

Standard Build (Debug)
--------------------------------------------------------------------------------

Build with debug symbols and no optimizations (fast compile, slow runtime)::

    cargo build

Output location: ``target/debug/rax25kb``

Size: ~15-20 MB (includes debug symbols)

Use for: Development, debugging, testing

Release Build (Optimized)
--------------------------------------------------------------------------------

Build with full optimizations (slow compile, fast runtime)::

    cargo build --release

Output location: ``target/release/rax25kb``

Size: ~2-3 MB (stripped of debug symbols on most platforms)

Use for: Production deployment, performance testing

Optimized Release Build
--------------------------------------------------------------------------------

Maximum optimization and size reduction::

    cargo build --release
    strip target/release/rax25kb  # Remove remaining symbols (Linux/macOS)

Final size: ~1-2 MB

Build Verification
--------------------------------------------------------------------------------

Check that the binary was built successfully::

    ls -lh target/release/rax25kb

Run basic tests::

    target/release/rax25kb --help

Expected output::

    rax25kb v2.0.0 - Multi-Port Cross-Connect KISS Bridge
    
    Usage: rax25kb [OPTIONS]
    ...

================================================================================
Build Options and Features
================================================================================

Cargo Build Flags
--------------------------------------------------------------------------------

**Common flags**::

    --release              # Enable optimizations
    --target <triple>      # Cross-compile for different platform
    --verbose              # Show detailed build output
    --jobs <N>             # Use N parallel jobs (default: # of CPUs)
    --target-dir <dir>     # Place build artifacts in <dir>

**Example: Verbose release build**::

    cargo build --release --verbose

Cross-Compilation
--------------------------------------------------------------------------------

Build for a different target platform::

    # Add target
    rustup target add armv7-unknown-linux-gnueabihf

    # Build for ARM
    cargo build --release --target armv7-unknown-linux-gnueabihf

**Common targets**:

* ``x86_64-unknown-linux-gnu`` - 64-bit Linux
* ``x86_64-pc-windows-msvc`` - 64-bit Windows
* ``x86_64-apple-darwin`` - 64-bit macOS (Intel)
* ``aarch64-apple-darwin`` - 64-bit macOS (Apple Silicon)
* ``aarch64-unknown-linux-gnu`` - 64-bit ARM Linux (Raspberry Pi 4+)
* ``armv7-unknown-linux-gnueabihf`` - 32-bit ARM Linux (Raspberry Pi 2/3)

Cross-compilation may require additional toolchain setup.

Static Linking (Linux)
--------------------------------------------------------------------------------

Create a fully static binary with musl libc::

    # Install musl target
    rustup target add x86_64-unknown-linux-musl

    # Build static binary
    cargo build --release --target x86_64-unknown-linux-musl

Benefits:

* No runtime dependencies
* Portable across Linux distributions
* Slightly larger binary size

LTO (Link-Time Optimization)
--------------------------------------------------------------------------------

Enable aggressive optimization in ``Cargo.toml``::

    [profile.release]
    lto = true
    codegen-units = 1
    opt-level = 3

Then build::

    cargo build --release

Result:

* Smaller binary size
* Better runtime performance
* Significantly longer compile time

================================================================================
Development Builds
================================================================================

Development Workflow
--------------------------------------------------------------------------------

**1. Make code changes**

Edit source files in ``src/``

**2. Quick check (no build)**::

    cargo check

Faster than building, catches compilation errors only

**3. Build and test**::

    cargo build
    cargo test

**4. Run directly**::

    cargo run -- -c config.cfg

The ``--`` separates cargo arguments from program arguments

Incremental Compilation
--------------------------------------------------------------------------------

Cargo uses incremental compilation by default. Only changed files are
recompiled. To force full rebuild::

    cargo clean
    cargo build

Watch Mode (Auto-rebuild)
--------------------------------------------------------------------------------

Install cargo-watch::

    cargo install cargo-watch

Auto-rebuild on file changes::

    cargo watch -x build
    cargo watch -x 'run -- -c config.cfg'  # Auto-run on changes

================================================================================
Testing
================================================================================

Running Tests
--------------------------------------------------------------------------------

**Run all tests**::

    cargo test

**Run tests with output**::

    cargo test -- --nocapture

**Run specific test**::

    cargo test test_name

**Run doc tests only**::

    cargo test --doc

Test Coverage
--------------------------------------------------------------------------------

Install tarpaulin (Linux only)::

    cargo install cargo-tarpaulin

Generate coverage report::

    cargo tarpaulin --out Html

Output: ``tarpaulin-report.html``

Linting and Formatting
--------------------------------------------------------------------------------

**Check formatting**::

    cargo fmt --check

**Apply formatting**::

    cargo fmt

**Run clippy linter**::

    cargo clippy

**Strict linting**::

    cargo clippy -- -D warnings

================================================================================
Installation
================================================================================

Install from Source
--------------------------------------------------------------------------------

Build and install to ``~/.cargo/bin/``::

    cargo install --path .

Install to custom location::

    cargo build --release
    sudo cp target/release/rax25kb /usr/local/bin/
    sudo chmod 755 /usr/local/bin/rax25kb

System-Wide Installation (Linux)
--------------------------------------------------------------------------------

**1. Build the binary**::

    cargo build --release

**2. Install binary**::

    sudo install -m 755 target/release/rax25kb /usr/local/bin/

**3. Install configuration**::

    sudo mkdir -p /etc/rax25kb
    sudo cp examples/rax25kb.cfg /etc/rax25kb/

**4. Install systemd service** (optional)::

    sudo cp systemd/rax25kb.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable rax25kb
    sudo systemctl start rax25kb

Uninstallation
--------------------------------------------------------------------------------

**If installed via cargo**::

    cargo uninstall rax25kb

**If installed manually**::

    sudo rm /usr/local/bin/rax25kb
    sudo rm -rf /etc/rax25kb
    sudo systemctl stop rax25kb
    sudo systemctl disable rax25kb
    sudo rm /etc/systemd/system/rax25kb.service

================================================================================
Troubleshooting Build Issues
================================================================================

Common Build Errors
--------------------------------------------------------------------------------

Error: "linker 'cc' not found"
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Cause**: C compiler not installed

**Solution**:

* **Linux**: ``sudo apt install build-essential``
* **macOS**: ``xcode-select --install``
* **Windows**: Install Visual Studio Build Tools

Error: "failed to run custom build command for serialport"
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Cause**: Missing libudev development files (Linux)

**Solution**::

    # Debian/Ubuntu
    sudo apt install libudev-dev

    # Fedora/RHEL
    sudo dnf install systemd-devel

Error: "could not find Cargo.toml"
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Cause**: Not in project root directory

**Solution**::

    cd /path/to/rax25kb
    cargo build

Error: "rustc version too old"
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Cause**: Rust toolchain older than required (< 1.70)

**Solution**::

    rustup update stable

Error: "failed to fetch" or network timeouts
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Cause**: Network issues or firewall blocking crates.io

**Solution**:

* Check internet connection
* Configure cargo to use a proxy if needed
* Try alternate cargo registry mirror

Cargo Cache Issues
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

If builds fail mysteriously, clear cargo cache::

    rm -rf ~/.cargo/registry
    rm -rf ~/.cargo/git
    cargo clean

Then rebuild.

Platform-Specific Issues
--------------------------------------------------------------------------------

Windows: "error: linker link.exe not found"
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Install Visual Studio Build Tools with C++ support:

https://visualstudio.microsoft.com/downloads/

macOS: "xcrun: error: invalid active developer path"
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Install Xcode Command Line Tools::

    xcode-select --install

Linux: "error while loading shared libraries: libudev.so.1"
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

At runtime, install udev::

    sudo apt install libudev1  # Debian/Ubuntu
    sudo dnf install systemd-libs  # Fedora/RHEL

Performance Issues
--------------------------------------------------------------------------------

Slow Compilation
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

* Use ``cargo check`` instead of ``cargo build`` during development
* Use more parallel jobs: ``cargo build -j 8``
* Disable LTO for debug builds
* Use faster linker (mold on Linux, lld on other platforms)

Large Binary Size
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

* Use release build: ``cargo build --release``
* Strip symbols: ``strip target/release/rax25kb``
* Enable LTO: add ``lto = true`` to ``Cargo.toml`` release profile
* Consider UPX compression (not always recommended)

================================================================================
Building Documentation
================================================================================

API Documentation
--------------------------------------------------------------------------------

Build and open HTML documentation::

    cargo doc --open

Output location: ``target/doc/rax25kb/index.html``

Build without opening::

    cargo doc --no-deps

Man Pages
--------------------------------------------------------------------------------

If man pages are provided::

    # Install man pages
    sudo mkdir -p /usr/local/share/man/man1
    sudo mkdir -p /usr/local/share/man/man5
    sudo cp docs/rax25kb.1 /usr/local/share/man/man1/
    sudo cp docs/rax25kb.cfg.5 /usr/local/share/man/man5/
    sudo mandb  # Update man page database

================================================================================
Continuous Integration
================================================================================

GitHub Actions
--------------------------------------------------------------------------------

The project includes ``.github/workflows/ci.yml`` for automated builds.

Builds test:

* Multiple platforms (Linux, macOS, Windows)
* Multiple Rust versions (stable, beta, nightly)
* Clippy lints
* Format checking
* Test suite

Local CI Simulation
--------------------------------------------------------------------------------

Run the same checks as CI::

    # Format check
    cargo fmt --check

    # Lint
    cargo clippy -- -D warnings

    # Build
    cargo build --release

    # Test
    cargo test

================================================================================
Building for Embedded Systems
================================================================================

Raspberry Pi
--------------------------------------------------------------------------------

**Native build on Pi**::

    # Install Rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

    # Build
    cargo build --release

**Cross-compile from x86_64**::

    # For Pi 4+ (64-bit)
    rustup target add aarch64-unknown-linux-gnu
    cargo build --release --target aarch64-unknown-linux-gnu

    # For Pi 2/3 (32-bit)
    rustup target add armv7-unknown-linux-gnueabihf
    cargo build --release --target armv7-unknown-linux-gnueabihf

OpenWrt / Embedded Linux
--------------------------------------------------------------------------------

Cross-compile for OpenWrt targets using the OpenWrt SDK.

See: https://openwrt.org/docs/guide-developer/toolchain/use-buildsystem

================================================================================
Distribution Packages
================================================================================

Creating Distribution Packages
--------------------------------------------------------------------------------

**Debian/Ubuntu (.deb)**

Use ``cargo-deb``::

    cargo install cargo-deb
    cargo deb

Output: ``target/debian/rax25kb_*.deb``

**RPM-based (.rpm)**

Use ``cargo-rpm``::

    cargo install cargo-rpm
    cargo rpm build

**Arch Linux (AUR)**

See: https://wiki.archlinux.org/title/Rust_package_guidelines

**Homebrew (macOS)**

Create a Homebrew formula. See: https://docs.brew.sh/Formula-Cookbook

================================================================================
Verifying the Build
================================================================================

Binary Verification
--------------------------------------------------------------------------------

**Check binary type**::

    file target/release/rax25kb

Expected: ELF 64-bit LSB executable (Linux) or similar for your platform

**Check dependencies**::

    # Linux
    ldd target/release/rax25kb

    # macOS
    otool -L target/release/rax25kb

**Check size**::

    ls -lh target/release/rax25kb

Expected: 1-3 MB for release build

Security Verification
--------------------------------------------------------------------------------

**Check for hardening flags** (Linux)::

    hardening-check target/release/rax25kb

**Scan for vulnerabilities**::

    cargo audit

Install cargo-audit::

    cargo install cargo-audit

================================================================================
Getting Help
================================================================================

Build Problems
--------------------------------------------------------------------------------

1. Check this documentation
2. Search GitHub issues: https://github.com/ke4ahr/rax25kb/issues
3. Ask on amateur radio forums
4. File a bug report with:

   * Operating system and version
   * Rust version (``rustc --version``)
   * Full error message
   * Steps to reproduce

Resources
--------------------------------------------------------------------------------

* **Rust Book**: https://doc.rust-lang.org/book/
* **Cargo Book**: https://doc.rust-lang.org/cargo/
* **Rust Community**: https://www.rust-lang.org/community
* **Project Issues**: https://github.com/ke4ahr/rax25kb/issues

================================================================================
End of Building Documentation
================================================================================
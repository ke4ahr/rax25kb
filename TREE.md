# rax25kb File Tree

This document describes the directory structure of the rax25kb project.

## Project Structure

```
rax25kb/
├── ARCHITECTURE.md          # System architecture documentation
├── CHANGELOG.md             # Version history and changes
├── Cargo.toml              # Rust package manifest
├── INSTALL.md              # Installation instructions
├── LICENSE                 # GPL-3.0 license file
├── README.md               # Project overview
├── TREE.md                 # This file
│
├── src/                    # Source code directory
│   └── main.rs            # Main program source code
│
├── doc/                    # Documentation directory
│   ├── source/            # Sphinx documentation source
│   │   ├── conf.py        # Sphinx configuration
│   │   ├── index.rst      # Documentation index
│   │   ├── introduction.rst   # Introduction
│   │   ├── installation.rst   # Installation guide
│   │   ├── configuration.rst  # Configuration guide
│   │   ├── usage.rst          # Usage guide
│   │   ├── windows.rst        # Windows-specific guide
│   │   ├── cross-connects.rst # Cross-connect guide
│   │   ├── kiss-xkiss.rst     # KISS/XKISS translation
│   │   ├── architecture.rst   # Architecture docs
│   │   ├── troubleshooting.rst # Troubleshooting
│   │   ├── api.rst            # API reference
│   │   ├── contributing.rst   # Contribution guide
│   │   ├── changelog.rst      # Changelog in RST
│   │   ├── _static/           # Static files (CSS, images)
│   │   └── _templates/        # Custom templates
│   │
│   └── examples/          # Example configuration files
│       ├── single-tnc.cfg      # Single TNC example
│       ├── multi-tnc.cfg       # Multiple TNC example
│       ├── windows.cfg         # Windows example
│       └── kiss-xkiss.cfg      # KISS/XKISS translation example
│
├── man/                    # Manual pages
│   ├── rax25kb.1          # Man page for rax25kb command (section 1)
│   └── rax25kb.cfg.5      # Man page for config file (section 5)
│
└── target/                 # Build output (created by cargo)
    ├── debug/             # Debug builds
    └── release/           # Release builds

```

## Directory Descriptions

### Root Directory

- **ARCHITECTURE.md**: Detailed technical architecture documentation
- **CHANGELOG.md**: Version history with detailed changes per release
- **Cargo.toml**: Rust package configuration and dependencies
- **INSTALL.md**: Comprehensive installation guide for all platforms
- **LICENSE**: GNU General Public License v3.0 or later
- **README.md**: Project overview, quick start, and links
- **TREE.md**: This file describing the project structure

### src/

Contains the Rust source code:

- **main.rs**: Complete application source code including:
  - Configuration parsing
  - Cross-connect bridge implementation
  - KISS/XKISS protocol handling
  - Serial port management
  - TCP server implementation
  - Logging system
  - PCAP capture
  - PhilFlag correction

### doc/

Documentation directory with two main subdirectories:

#### doc/source/

Sphinx documentation source files in reStructuredText format:

- **conf.py**: Sphinx configuration (theme, extensions, etc.)
- **index.rst**: Main documentation index
- **introduction.rst**: Project introduction and overview
- **installation.rst**: Platform-specific installation instructions
- **configuration.rst**: Configuration file format and options
- **usage.rst**: Usage examples and common scenarios
- **windows.rst**: Windows-specific usage guide
- **cross-connects.rst**: Cross-connect configuration guide
- **kiss-xkiss.rst**: KISS/XKISS translation documentation
- **architecture.rst**: Detailed architecture documentation
- **troubleshooting.rst**: Common issues and solutions
- **api.rst**: API reference (if applicable)
- **contributing.rst**: Contribution guidelines
- **changelog.rst**: Changelog in reStructuredText format

#### doc/examples/

Example configuration files for common use cases:

- **single-tnc.cfg**: Basic single TNC bridge
- **multi-tnc.cfg**: Multiple TNC bridges
- **windows.cfg**: Windows-specific configuration
- **kiss-xkiss.cfg**: KISS to XKISS translation example

### man/

Unix manual pages in troff format:

- **rax25kb.1**: Man page for the rax25kb command (section 1 - user commands)
- **rax25kb.cfg.5**: Man page for the configuration file (section 5 - file formats)

These can be installed system-wide and viewed with `man rax25kb` and `man rax25kb.cfg`.

### target/

Build output directory created by Cargo (not included in repository):

- **debug/**: Debug builds for development
- **release/**: Optimized release builds
- Contains compiled binaries and intermediate build artifacts

## File Types

### Rust Source Files (.rs)
- Written in Rust programming language
- Compiled by cargo build system
- Located in src/ directory

### Documentation Files (.rst)
- reStructuredText format
- Processed by Sphinx documentation generator
- Located in doc/source/

### Configuration Files (.cfg)
- INI-style format
- Key=value pairs
- Support comments with # character
- Examples in doc/examples/

### Manual Pages (.1, .5)
- Troff/groff format
- Section 1 for commands, section 5 for file formats
- Located in man/

### Markdown Files (.md)
- Markdown format
- Project documentation
- Located in root directory

## Build Artifacts

The following directories are created during build and are not tracked in version control:

- **target/**: Cargo build output
- **doc/build/**: Sphinx documentation build output (HTML, PDF, etc.)

## Configuration File Locations

### Default Search Paths

rax25kb searches for configuration files in these locations (in order):

1. File specified with `-c` option
2. `./rax25kb.cfg` (current directory)
3. `/etc/rax25kb/rax25kb.cfg` (Linux)
4. `/usr/local/etc/rax25kb.cfg` (macOS)
5. `C:\Program Files\rax25kb\rax25kb.cfg` (Windows)

### Log File Locations

Default log file locations:

- Linux: `/var/log/rax25kb.log`
- macOS: `/var/log/rax25kb.log` or `/usr/local/var/log/rax25kb.log`
- Windows: `C:\ProgramData\rax25kb\rax25kb.log`

### PID File Locations

Default PID file locations:

- Linux: `/var/run/rax25kb.pid` or `/run/rax25kb.pid`
- macOS: `/var/run/rax25kb.pid`
- Windows: `C:\ProgramData\rax25kb\rax25kb.pid`

## Building Documentation

### Sphinx HTML Documentation

```bash
cd doc
sphinx-build -b html source build/html
```

Output: `doc/build/html/index.html`

### Sphinx PDF Documentation

```bash
cd doc
sphinx-build -b latex source build/latex
cd build/latex
make
```

Output: `doc/build/latex/rax25kb.pdf`

### Man Pages

Man pages are pre-built in the `man/` directory and can be installed directly:

```bash
sudo install -m 644 man/rax25kb.1 /usr/local/share/man/man1/
sudo install -m 644 man/rax25kb.cfg.5 /usr/local/share/man/man5/
sudo mandb  # Update man database
```

## Version Control

### Included in Repository

- All source code (src/)
- Documentation source (doc/source/)
- Man pages (man/)
- Example configurations (doc/examples/)
- Project documentation (*.md files)
- Build configuration (Cargo.toml)

### Excluded from Repository

- Build artifacts (target/)
- Documentation builds (doc/build/)
- Log files
- PID files
- User configuration files
- Environment directory (env/)

## Contributing

When contributing to the project, please:

1. Follow the existing directory structure
2. Place new examples in `doc/examples/`
3. Update documentation in `doc/source/` as needed
4. Update CHANGELOG.md for significant changes
5. Ensure man pages reflect any CLI changes

See CONTRIBUTING.md for detailed contribution guidelines.

## License

All files in this project are licensed under GPL-3.0-or-later unless otherwise specified.

See LICENSE file for complete license text.

================================================================================
Contributing Guide
================================================================================

rax25kb - AX.25 KISS Bridge with Multi-Port Cross-Connect Support

:Version: 2.0.0
:Author: Kris Kirby, KE4AHR
:Date: December 2025

================================================================================
Welcome Contributors!
================================================================================

Thank you for your interest in contributing to rax25kb! This project welcomes
contributions from amateur radio operators, software developers, and anyone
interested in packet radio technology.

All contributors are expected to follow our Code of Conduct (see below).

================================================================================
Ways to Contribute
================================================================================

Code Contributions
--------------------------------------------------------------------------------

* Bug fixes
* New features
* Performance improvements
* Code refactoring and cleanup
* Platform-specific improvements

Documentation
--------------------------------------------------------------------------------

* Improving existing documentation
* Writing tutorials and guides
* Adding code comments
* Creating examples
* Translating documentation

Testing
--------------------------------------------------------------------------------

* Testing on different platforms
* Testing with different TNC hardware
* Reporting bugs with detailed reproduction steps
* Creating test cases

Community Support
--------------------------------------------------------------------------------

* Answering questions on GitHub issues
* Helping other users in forums
* Sharing your setup and configurations
* Writing blog posts and articles

Hardware Testing
--------------------------------------------------------------------------------

* Testing with various TNC models
* Documenting TNC-specific quirks
* Reporting compatibility issues
* Contributing hardware-specific fixes

================================================================================
Getting Started
================================================================================

First Steps
--------------------------------------------------------------------------------

1. **Fork the repository** on GitHub
2. **Clone your fork**::

    git clone https://github.com/YOUR_USERNAME/rax25kb.git
    cd rax25kb

3. **Add upstream remote**::

    git remote add upstream https://github.com/ke4ahr/rax25kb.git

4. **Create a feature branch**::

    git checkout -b feature/your-feature-name

5. **Install development tools** (see Development Environment below)

Development Environment Setup
--------------------------------------------------------------------------------

**Required Tools**

* Rust toolchain (latest stable)
* Git
* Text editor or IDE (VS Code, Vim, Emacs, etc.)

**Recommended Tools**

* ``cargo-watch`` - Auto-rebuild on file changes
* ``cargo-edit`` - Edit Cargo.toml from command line
* ``cargo-audit`` - Security vulnerability scanning
* ``rustfmt`` - Code formatter (included with Rust)
* ``clippy`` - Linter (included with Rust)

Install recommended tools::

    cargo install cargo-watch cargo-edit cargo-audit

**IDE Setup**

For VS Code, install:

* rust-analyzer extension
* Better TOML extension
* CodeLLDB (for debugging)

================================================================================
Development Workflow
================================================================================

Making Changes
--------------------------------------------------------------------------------

1. **Update your fork**::

    git fetch upstream
    git checkout main
    git merge upstream/main

2. **Create a feature branch**::

    git checkout -b feature/my-new-feature

3. **Make your changes**

   * Write clear, concise code
   * Follow the coding style (see Style Guide below)
   * Add tests for new functionality
   * Update documentation as needed

4. **Test your changes**::

    cargo test
    cargo clippy
    cargo fmt --check

5. **Commit your changes**::

    git add .
    git commit -m "feat: add support for XYZ"

   Follow commit message conventions (see below)

6. **Push to your fork**::

    git push origin feature/my-new-feature

7. **Create a Pull Request**

   * Go to GitHub and create a PR from your branch
   * Fill out the PR template
   * Link any related issues

Testing Your Changes
--------------------------------------------------------------------------------

**Run unit tests**::

    cargo test

**Run tests with output**::

    cargo test -- --nocapture

**Run linter**::

    cargo clippy -- -D warnings

**Check formatting**::

    cargo fmt --check

**Apply formatting**::

    cargo fmt

**Build release binary**::

    cargo build --release

**Manual testing**::

    target/release/rax25kb -c your_test.cfg

Continuous Integration
--------------------------------------------------------------------------------

All pull requests run automated checks:

* Build on Linux, macOS, Windows
* Run test suite
* Check formatting (``cargo fmt``)
* Run linter (``cargo clippy``)
* Check for security vulnerabilities

Make sure your code passes these checks before submitting a PR.

================================================================================
Code Style Guide
================================================================================

Rust Style
--------------------------------------------------------------------------------

We follow the **official Rust style guide** with these conventions:

**Formatting**

* Use ``rustfmt`` with default settings
* Run ``cargo fmt`` before committing
* 100-character line limit (rustfmt default)
* 4-space indentation

**Naming Conventions**

* Types: ``UpperCamelCase`` (e.g., ``SerialPortManager``)
* Functions: ``snake_case`` (e.g., ``open_port``)
* Constants: ``UPPER_SNAKE_CASE`` (e.g., ``KISS_FEND``)
* Variables: ``snake_case`` (e.g., ``frame_buffer``)

**Documentation**

* Public items must have doc comments (``///``)
* Use proper Markdown formatting in doc comments
* Include examples for complex functions
* Document parameters and return values

Code Organization
--------------------------------------------------------------------------------

**Module Structure**::

    src/
    ├── main.rs           # Main entry point (assembled from parts)
    ├── main_part1.rs     # Core types and protocols
    ├── main_part2.rs     # Configuration parsing
    ├── main_part3.rs     # Logging and KISS handling
    ├── main_part4.rs     # Serial and cross-connect management
    └── main_part5.rs     # Main function and initialization

**Adding New Features**

* Keep functions focused and small (< 100 lines preferred)
* Use descriptive names that explain purpose
* Extract complex logic into helper functions
* Add comments for non-obvious code

**Error Handling**

* Use ``Result<T, E>`` for operations that can fail
* Use ``Option<T>`` for optional values
* Propagate errors with ``?`` operator
* Provide helpful error messages

**Threading**

* Document thread safety explicitly
* Use ``Arc<Mutex<T>>`` for shared mutable state
* Minimize lock scope to prevent deadlocks
* Document which threads access which data

Comments
--------------------------------------------------------------------------------

**When to Comment**

* Complex algorithms or logic
* Non-obvious workarounds or hacks
* Protocol-specific behavior
* Hardware-specific quirks
* Performance-critical code

**When Not to Comment**

* Obvious code (``// increment counter``)
* Restating what code does (``// call function``)
* Outdated or inaccurate comments (update or remove)

**Example**::

    // PhilFlag fixes KISS protocol bugs in TASCO modem chipsets
    // by escaping unescaped FEND bytes in the data payload.
    // Without this, TASCO modems misinterpret frame boundaries.
    fn process_frame_with_phil_flag(frame: &[u8]) -> Vec<u8> {
        // ...
    }

================================================================================
Commit Message Guidelines
================================================================================

Format
--------------------------------------------------------------------------------

We use **Conventional Commits** format::

    <type>(<scope>): <subject>

    <body>

    <footer>

**Type** (required)

* ``feat`` - New feature
* ``fix`` - Bug fix
* ``docs`` - Documentation only changes
* ``style`` - Code style changes (formatting, etc.)
* ``refactor`` - Code refactoring
* ``perf`` - Performance improvements
* ``test`` - Adding or updating tests
* ``chore`` - Build process, tooling, dependencies

**Scope** (optional)

* ``serial`` - Serial port handling
* ``tcp`` - TCP networking
* ``kiss`` - KISS protocol
* ``ax25`` - AX.25 protocol
* ``config`` - Configuration
* ``log`` - Logging
* ``pcap`` - PCAP capture

**Examples**::

    feat(kiss): add support for Extended KISS port numbers

    fix(serial): correct baud rate calculation for Kenwood TS-2000

    docs: update building instructions for Windows

    refactor(tcp): simplify connection handler logic

    perf(kiss): reduce frame buffer allocations

Breaking Changes
--------------------------------------------------------------------------------

For breaking changes, add ``BREAKING CHANGE:`` in the footer::

    feat(config): change configuration file format

    BREAKING CHANGE: Configuration files now use TOML instead of
    key=value format. Users must convert existing configs.

Multiple Changes
--------------------------------------------------------------------------------

If a commit addresses multiple changes, consider splitting it::

    # Instead of:
    git commit -m "fix bugs and add feature"

    # Do:
    git add file1.rs
    git commit -m "fix(serial): handle timeout errors correctly"
    git add file2.rs
    git commit -m "feat(kiss): add frame validation"

================================================================================
Pull Request Process
================================================================================

Before Submitting
--------------------------------------------------------------------------------

1. **Ensure all tests pass**::

    cargo test

2. **Run linter**::

    cargo clippy -- -D warnings

3. **Format code**::

    cargo fmt

4. **Update documentation** if needed

5. **Add entry to CHANGELOG.rst** (see below)

6. **Rebase on latest main**::

    git fetch upstream
    git rebase upstream/main

PR Title and Description
--------------------------------------------------------------------------------

**Title Format**::

    <type>: brief description (< 72 chars)

**Description Should Include**:

* What changes were made
* Why the changes were needed
* How to test the changes
* Any breaking changes
* Related issue numbers (``Fixes #123``)

**Example**::

    fix: handle serial port disconnect gracefully

    Previously, serial port disconnection would cause the entire
    application to crash. This change handles disconnection errors
    and logs them appropriately while keeping other connections alive.

    Testing:
    1. Start rax25kb with a USB serial adapter
    2. Unplug the adapter while running
    3. Verify the program continues running and logs the error

    Fixes #42

Review Process
--------------------------------------------------------------------------------

1. **Automated checks** must pass (CI build, tests, linting)
2. **Code review** by maintainers
3. **Discussion** and potential changes requested
4. **Approval** and merge

**Tips for Faster Review**

* Keep PRs focused (one feature/fix per PR)
* Write clear commit messages and PR descriptions
* Respond promptly to feedback
* Be open to suggestions and changes

After PR is Merged
--------------------------------------------------------------------------------

1. **Delete your feature branch**::

    git branch -d feature/my-feature
    git push origin --delete feature/my-feature

2. **Update your fork**::

    git checkout main
    git fetch upstream
    git merge upstream/main
    git push origin main

3. **Thank the reviewers!** 🎉

================================================================================
Changelog Guidelines
================================================================================

When making changes, add an entry to ``CHANGELOG.rst``:

Format
--------------------------------------------------------------------------------

::

    Version X.Y.Z (YYYY-MM-DD)
    ================================================================================

    Added
    --------------------------------------------------------------------------------
    * New feature description

    Changed
    --------------------------------------------------------------------------------
    * Changed behavior description

    Fixed
    --------------------------------------------------------------------------------
    * Bug fix description

    Deprecated
    --------------------------------------------------------------------------------
    * Deprecated feature description

    Removed
    --------------------------------------------------------------------------------
    * Removed feature description

    Security
    --------------------------------------------------------------------------------
    * Security fix description

Guidelines
--------------------------------------------------------------------------------

* Add entries to the **Unreleased** section
* Use present tense ("Add feature" not "Added feature")
* Reference issue/PR numbers when applicable
* Keep entries brief but descriptive
* Group related changes together

================================================================================
Issue Reporting
================================================================================

Bug Reports
--------------------------------------------------------------------------------

When reporting bugs, include:

**Environment**

* Operating system and version
* Rust version (``rustc --version``)
* rax25kb version
* Hardware (TNC model, serial adapter)

**Description**

* What you expected to happen
* What actually happened
* Steps to reproduce
* Error messages or logs

**Example Bug Report**::

    **Environment:**
    - OS: Ubuntu 22.04 LTS
    - Rust: 1.75.0
    - rax25kb: 2.0.0
    - TNC: Kantronics KPC-3+

    **Description:**
    Serial port fails to open with "Permission denied" error.

    **Steps to Reproduce:**
    1. Configure serial_port0000=/dev/ttyUSB0
    2. Run: rax25kb -c config.cfg
    3. Error appears immediately

    **Expected:**
    Serial port opens successfully

    **Actual:**
    Error: Permission denied (os error 13)

    **Logs:**
    [2025-12-24 10:00:00] [ERROR] Failed to open serial port

Feature Requests
--------------------------------------------------------------------------------

When requesting features, include:

* **Use case**: Why you need this feature
* **Description**: What the feature should do
* **Alternatives**: Other solutions you've considered
* **Examples**: Similar features in other software

**Example Feature Request**::

    **Feature:** Add IPv6 support for TCP endpoints

    **Use case:**
    My network uses IPv6-only addressing. Currently, rax25kb only
    supports IPv4 addresses in cross-connect definitions.

    **Description:**
    Allow IPv6 addresses in TCP endpoint specifications:
    tcp:[::1]:8001 or tcp:[2001:db8::1]:8001

    **Alternatives:**
    Using IPv4-to-IPv6 proxy, but native support would be cleaner.

Questions
--------------------------------------------------------------------------------

For questions:

* Check existing documentation first
* Search closed issues for similar questions
* Open a GitHub Discussion (not an issue)
* Be specific about what you're trying to do

================================================================================
Code Review Guidelines
================================================================================

For Reviewers
--------------------------------------------------------------------------------

**Be Constructive**

* Offer specific, actionable feedback
* Explain the "why" behind suggestions
* Acknowledge good code and improvements
* Be respectful and professional

**Focus On**

* Correctness and functionality
* Code clarity and maintainability
* Test coverage
* Documentation
* Performance implications
* Security concerns

**Review Checklist**

* [ ] Code follows project style guidelines
* [ ] Tests are included and pass
* [ ] Documentation is updated
* [ ] No obvious bugs or issues
* [ ] Changes are backwards compatible (or documented)
* [ ] Performance impact is acceptable
* [ ] Security implications are considered

For Contributors
--------------------------------------------------------------------------------

**Receiving Feedback**

* View feedback as helpful, not critical
* Ask for clarification if needed
* Don't take suggestions personally
* Thank reviewers for their time

**Responding to Feedback**

* Address all comments (even if "Acknowledged")
* Update code based on feedback
* Push changes to the same branch
* Reply to comments when done

================================================================================
Testing Guidelines
================================================================================

Writing Tests
--------------------------------------------------------------------------------

**Unit Tests**

Place tests in the same file as the code being tested::

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_kiss_port_extraction() {
            let frame = vec![0xC0, 0x00, 0x01, 0x02, 0xC0];
            let (port, cmd, _) = extract_kiss_info(&frame).unwrap();
            assert_eq!(port, 0);
            assert_eq!(cmd, 0);
        }
    }

**Integration Tests**

Place in ``tests/`` directory::

    // tests/serial_integration.rs
    #[test]
    fn test_serial_to_tcp_connection() {
        // Test full connection flow
    }

Test Coverage
--------------------------------------------------------------------------------

* Aim for >80% code coverage
* Test edge cases and error conditions
* Test platform-specific code on multiple platforms
* Include both positive and negative test cases

Manual Testing
--------------------------------------------------------------------------------

Before submitting, manually test:

* Fresh build from scratch
* Configuration file parsing
* Serial port detection
* TCP connections
* Cross-platform behavior (if possible)

================================================================================
Documentation Standards
================================================================================

Code Documentation
--------------------------------------------------------------------------------

**Public API**: Must have doc comments

::

    /// Opens a serial port with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Serial port configuration settings
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Port opened successfully
    /// * `Err(e)` - Failed to open port
    ///
    /// # Examples
    ///
    /// ```
    /// let config = SerialPortConfig { /* ... */ };
    /// manager.open_port(&config)?;
    /// ```
    fn open_port(&mut self, config: &SerialPortConfig) -> Result<(), Box<dyn std::error::Error>>

User Documentation
--------------------------------------------------------------------------------

* Use reStructuredText (.rst) format
* Follow existing documentation structure
* Include examples where appropriate
* Keep language clear and concise
* Consider non-native English speakers

Documentation Types
--------------------------------------------------------------------------------

* **README.md**: Project overview, quick start
* **architecture.rst**: System design and internals
* **building.rst**: Build instructions
* **contributing.rst**: This document
* **changelog.rst**: Version history
* **glossary.rst**: Term definitions
* **license.rst**: Licensing information

================================================================================
Code of Conduct
================================================================================

Our Pledge
--------------------------------------------------------------------------------

We pledge to make participation in our project a harassment-free experience
for everyone, regardless of:

* Age, body size, disability, ethnicity, gender identity and expression
* Level of experience, education, nationality, personal appearance
* Race, religion, or sexual identity and orientation
* Technical skill level or amateur radio license class

Expected Behavior
--------------------------------------------------------------------------------

* Use welcoming and inclusive language
* Be respectful of differing viewpoints and experiences
* Gracefully accept constructive criticism
* Focus on what is best for the community
* Show empathy towards other community members
* Help newcomers learn and contribute

Unacceptable Behavior
--------------------------------------------------------------------------------

* Harassment, trolling, or inflammatory comments
* Public or private harassment of any kind
* Publishing others' private information without permission
* Other conduct which could reasonably be considered inappropriate

Enforcement
--------------------------------------------------------------------------------

Instances of abusive, harassing, or otherwise unacceptable behavior may be
reported by contacting the project team. All complaints will be reviewed and
investigated promptly and fairly.

Project maintainers have the right and responsibility to remove, edit, or
reject comments, commits, code, issues, and other contributions that are not
aligned with this Code of Conduct.

================================================================================
License
================================================================================

By contributing to rax25kb, you agree that your contributions will be licensed
under the GNU General Public License v3.0 or later (GPL-3.0-or-later).

See ``LICENSE`` or ``license.rst`` for full license text.

================================================================================
Recognition
================================================================================

Contributors are recognized in:

* Git commit history
* ``CHANGELOG.rst`` for significant contributions
* Project README (for major contributors)
* Release notes

Your call sign and/or name will be attributed to your contributions unless you
request otherwise.

================================================================================
Getting Help
================================================================================

Questions About Contributing
--------------------------------------------------------------------------------

* Open a GitHub Discussion
* Ask in project issues (use "question" label)
* Contact project maintainer: Kris Kirby, KE4AHR

Resources
--------------------------------------------------------------------------------

* **Rust Book**: https://doc.rust-lang.org/book/
* **Git Tutorial**: https://git-scm.com/docs/gittutorial
* **Conventional Commits**: https://www.conventionalcommits.org/
* **GitHub Flow**: https://guides.github.com/introduction/flow/

================================================================================
Thank You!
================================================================================

Thank you for contributing to rax25kb! Your efforts help improve amateur
packet radio systems for everyone.

73 de KE4AHR

================================================================================
End of Contributing Guide
================================================================================
---
layout: default
title: Contributing
nav_order: 7
---

# Contributing to vexy_json

We welcome contributions to `vexy_json`! Whether it's bug reports, feature requests, documentation improvements, or code contributions, your help is greatly appreciated.

## How to Contribute

1.  **Fork the Repository**: Start by forking the `vexy_json` repository on GitHub.
2.  **Clone Your Fork**: Clone your forked repository to your local machine:
    ```bash
    git clone https://github.com/your-username/vexy_json.git
    cd vexy_json
    ```
3.  **Create a New Branch**: Create a new branch for your feature or bug fix:
    ```bash
    git checkout -b feature/your-feature-name
    # or
    git checkout -b bugfix/fix-description
    ```
4.  **Make Your Changes**: Implement your changes. Ensure your code adheres to the existing style and conventions.
5.  **Test Your Changes**: Run the test suite to ensure your changes haven't introduced any regressions and that new features are adequately covered.
    ```bash
    cargo test --all-features
    ```
6.  **Format and Lint**: Ensure your code is properly formatted and passes lint checks.
    ```bash
    cargo fmt
    cargo clippy --all-targets --all-features
    ```
7.  **Commit Your Changes**: Write clear and concise commit messages.
    ```bash
    git commit -m "feat: Add new feature X" # or "fix: Resolve bug Y"
    ```
8.  **Push to Your Fork**: Push your changes to your GitHub fork.
    ```bash
    git push origin feature/your-feature-name
    ```
9.  **Create a Pull Request**: Open a pull request from your fork to the `main` branch of the `vexy_json` repository. Provide a detailed description of your changes.

## Code Style and Conventions

-   Follow Rust's official style guidelines (enforced by `rustfmt`).
-   Use `clippy` to catch common mistakes and improve code quality.
-   Write clear and concise code comments and documentation where necessary.
-   Ensure new features have corresponding tests.

## Extending the Web Tool

If you're looking to contribute specifically to the `vexy_json` web tool, please refer to the [Developer Guide for Extending the Web Tool](developer-guide.md) for detailed information on its structure, build process, and development considerations.

## Reporting Bugs

If you find a bug, please open an issue on the [GitHub Issues page](https://github.com/twardoch/vexy_json/issues). When reporting a bug, please include:

-   A clear and concise description of the bug.
-   Steps to reproduce the behavior.
-   Expected behavior.
-   Actual behavior.
-   Any relevant error messages or stack traces.
-   Your Rust version (`rustc --version`).

## Feature Requests

Have an idea for a new feature? Open an issue on the [GitHub Issues page](https://github.com/twardoch/vexy_json/issues) to discuss it. Describe the feature, why you think it would be valuable, and any potential implementation details.

Thank you for contributing to `vexy_json`!

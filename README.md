# OS

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-MIT)
![Test Status](https://github.com/aicacia/rs-os/actions/workflows/test.yml/badge.svg)

## Quick Start

- `cp .env.example .env`
- `cp config.example.json config.json`
- `cargo run`
- https://petstore.swagger.io/?url=http://localhost:3000/oidc/api/openapi.json

## Architecture

The project is composed of modular Rust crates

### Each crate should be able to (unless it is only a shared library)

- Run independently as its own binary
- Packaged as a Docker image
- Used as a Rust library within other projects

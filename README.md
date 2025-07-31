# Stash

A place to put things.

## Installation

```bash
cargo install
```

## Setup

1. Generate a server keypair, and a client keypair

```bash
stash keygen
```

2. Set environment variables

```bash
# Path to SQLite DB. Will be created on demand
DATABASE_URL=...
# Root directory, in which data will be stored
STASH_ROOT=...
# Server secret key
STASH_SECRET_KEY=...
# Client public key
STASH_ADMIN=...
```

3. Start server

```bash
stash-daemon
```

## Usage

1. Set environment variables

```bash
# Client secret key
STASH_SECRET_KEY=...
# Server public key
STASH_SERVER=...
```

2. Run commands

```bash
stash help
```

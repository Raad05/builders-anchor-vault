# Anchor Vault

A Solana program built with Anchor that lets users securely deposit, withdraw, and manage SOL in a personal on-chain vault.

## Overview

Each user gets their own vault — a PDA-controlled system account — paired with a state account that stores the bump seeds needed to sign on behalf of the vault. Users can deposit and withdraw arbitrary amounts, or close the vault entirely to reclaim all lamports including rent.

## Program ID

`BU2EcRFU3RKpSCAFVGcBjmBUcQnrRjsHKnr97dMu9oYJ`

## Instructions

| Instruction  | Description                                                                                           |
| ------------ | ----------------------------------------------------------------------------------------------------- |
| `initialize` | Creates the `VaultState` PDA and derives the vault PDA for the caller                                 |
| `deposit`    | Transfers SOL from the user's wallet into their vault                                                 |
| `withdraw`   | Transfers SOL from the vault back to the user's wallet                                                |
| `close`      | Drains all SOL from the vault and closes the `VaultState` account, returning all lamports to the user |

## Account Structure

**`VaultState`** — PDA seeded with `["state", user_pubkey]`

- `vault_bump: u8` — bump seed for the vault PDA
- `state_bump: u8` — bump seed for this state account

**Vault** — System account PDA seeded with `["vault", vault_state_pubkey]`, holds the deposited SOL.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation)
- [Yarn](https://yarnpkg.com/)

## Getting Started

```bash
# Install dependencies
yarn install

# Build the program
anchor build

# Run tests
cargo test
```

## Testing

Tests use [LiteSVM](https://github.com/LiteSVM/litesvm) for fast, in-process simulation without a local validator. The test suite covers all four instructions end-to-end, including balance assertions for deposit, withdraw, and close.

```bash
cargo test
```

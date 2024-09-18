# Solana Test Vault

## Description

Solana Test Vault is a decentralized finance (DeFi) application built on the Solana blockchain. It allows users to deposit tokens into a vault, receive shares representing their deposit, and withdraw their tokens later. This project demonstrates the implementation of a basic vault system using Solana's programming model and the Anchor framework.

## Features

- Initialize a new vault
- Deposit tokens into the vault
- Receive shares proportional to the deposit
- Withdraw tokens from the vault
- Burn shares when withdrawing

## Prerequisites

Before you begin, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor](https://project-serum.github.io/anchor/getting-started/installation.html)
- [Node.js and npm](https://nodejs.org/en/download/)
- [Yarn](https://classic.yarnpkg.com/en/docs/install/)

## Installation and Setup

1. Clone the repository:
   ```
   git clone https://github.com/your-username/solana-test-vault.git
   cd solana-test-vault
   ```

2. Install dependencies:
   ```
   yarn install
   ```

3. Build the project:
   ```
   anchor build
   ```

4. Update the program ID in `lib.rs` and `Anchor.toml` with the new program ID generated during the build process.

**Note:** Depending on your operating system, you may need to update the `wallet` path in `Anchor.toml`. The default path is set for Unix-based systems. For Windows, you might need to change it to something like:

5. Run tests:
   ```
   anchor test
   ```

This command will execute all the tests defined in the `tests` directory, verifying the functionality of your Solana Test Vault program.


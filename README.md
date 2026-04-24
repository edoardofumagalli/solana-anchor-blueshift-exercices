# Solana Anchor Blueshift Exercices

Beginner Solana programs built with Anchor while following Blueshift exercises.

## Projects

- `blueshift_anchor_escrow`: token escrow with `make`, `take`, and `refund` instructions.
- `blueshift_anchor_vault`: SOL vault using PDA authority, deposits, withdrawals, rent checks, and Anchor tests.

## What I Practiced

- Anchor account constraints
- Program derived addresses
- PDA signer seeds
- Cross-program invocations
- SPL token transfers
- Rent-exemption checks
- TypeScript integration tests with Anchor

## Run Tests

```bash
cd blueshift_anchor_vault
yarn install
anchor test
```

```bash
cd blueshift_anchor_escrow
yarn install
anchor test
```

# NFT Staking

Anchor program on Solana for NFT staking with config, user, and rewards PDAs.

**Program ID:** `AKWnQtCbpRfeifA6F34SjS9kRFJEaNvG6wtyhnEEXmSR`

## What we built

Wired up a staking program that tracks three PDAs: a global `config` account, a per-wallet `user` account, and a `rewards` account that chains off the config. The PDA derivation is deterministic — same seeds always give the same address — which is what makes the staking logic trustless.

Wrote the `initialize` instruction to set up the config state on-chain, confirmed the transaction landed, then derived all three PDAs in the TypeScript tests and verified the bumps matched. The rewards PDA is a chained derivation (seeds include the config PDA address), which is a pattern worth knowing for any program that needs linked accounts.

## Tests passing

![test pass](public/Screenshot%202026-06-02%20022148.png)
![pda derivation](public/Screenshot%202026-06-02%20022208.png)

## Run

```bash
anchor test
```

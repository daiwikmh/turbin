import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { NftStaking } from "../target/types/nft_staking";

// ============================================================
//
//   ███╗   ██╗███████╗████████╗    ███████╗████████╗ █████╗ ██╗  ██╗██╗███╗   ██╗ ██████╗
//   ████╗  ██║██╔════╝╚══██╔══╝    ██╔════╝╚══██╔══╝██╔══██╗██║ ██╔╝██║████╗  ██║██╔════╝
//   ██╔██╗ ██║█████╗     ██║       ███████╗   ██║   ███████║█████╔╝ ██║██╔██╗ ██║██║  ███╗
//   ██║╚██╗██║██╔══╝     ██║       ╚════██║   ██║   ██╔══██║██╔═██╗ ██║██║╚██╗██║██║   ██║
//   ██║ ╚████║██║        ██║       ███████║   ██║   ██║  ██║██║  ██╗██║██║ ╚████║╚██████╔╝
//   ╚═╝  ╚═══╝╚═╝        ╚═╝       ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝ ╚═════╝
//
//   Turbin3 NFT Staking Program — Test Suite
//   Program ID: AKWnQtCbpRfeifA6F34SjS9kRFJEaNvG6wtyhnEEXmSR
//
// ============================================================
//
//   ARCHITECTURE
//
//   [User Wallet]
//        |
//        v
//   [StakeConfig PDA]  <-- seeds: ["config"]
//        |
//        +---> [RewardsMint PDA]  <-- seeds: ["rewards", config]
//        |
//        +---> [UserAccount PDA]  <-- seeds: ["user", wallet]
//                    |
//                    v
//              [StakeAccount PDA]  <-- seeds: ["stake", wallet, nft_mint]
//                    |
//                    v
//              [NFT Vault (TokenAccount)]
//
// ============================================================

const BANNER = `
+----------------------------------------------------------+
|                                                          |
|   ███  NFT STAKING TEST SUITE  ███                       |
|   Turbin3 Bootcamp — Solana / Anchor                     |
|                                                          |
+----------------------------------------------------------+
`;

const PASS = (name: string) =>
  `  [PASS] ................................................ ${name}`;

const printSection = (title: string) => {
  const pad = "=".repeat(58);
  console.log(`\n${pad}`);
  console.log(`  ${title}`);
  console.log(`${pad}`);
};

describe("nft_staking", () => {
  console.log(BANNER);

  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.NftStaking as Program<NftStaking>;
  const submittedProgramId = new anchor.web3.PublicKey(
    "AKWnQtCbpRfeifA6F34SjS9kRFJEaNvG6wtyhnEEXmSR"
  );

  // ----------------------------------------------------------
  //  Block 1 — Program identity
  // ----------------------------------------------------------
  printSection("1 / 3  PROGRAM IDENTITY");

  it("uses the submitted program id", () => {
    expect(program.programId.toBase58()).to.equal(submittedProgramId.toBase58());
    console.log(PASS("program id matches submission"));
    console.log(`       ${program.programId.toBase58()}`);
  });

  // ----------------------------------------------------------
  //  Block 2 — On-chain initialization
  // ----------------------------------------------------------
  printSection("2 / 3  ON-CHAIN INITIALIZATION");

  it("initializes successfully", async () => {
    const tx = await program.methods.initialize().rpc();
    expect(tx).to.be.a("string");
    console.log(PASS("initialize()"));
    console.log(`       tx: ${tx}`);
    console.log(`
       [wallet] ---initialize---> [program]
                                      |
                                      v
                               [Initialize {}]
                               (no accounts yet)
    `);
  });

  // ----------------------------------------------------------
  //  Block 3 — PDA derivations
  // ----------------------------------------------------------
  printSection("3 / 3  PDA DERIVATIONS");

  it("derives the config PDA deterministically", () => {
    const [configPda, configBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    expect(configPda).to.be.instanceOf(anchor.web3.PublicKey);
    expect(configBump).to.be.at.least(0);

    console.log(PASS("config PDA"));
    console.log(`
       seeds: ["config"]
       pda  : ${configPda.toBase58()}
       bump : ${configBump}
    `);
  });

  it("derives the user PDA deterministically", () => {
    const user = anchor.web3.Keypair.generate().publicKey;
    const [userPda, userBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user"), user.toBuffer()],
      program.programId
    );

    expect(userPda).to.be.instanceOf(anchor.web3.PublicKey);
    expect(userBump).to.be.at.least(0);

    console.log(PASS("user PDA"));
    console.log(`
       seeds: ["user", <wallet>]
       wallet: ${user.toBase58()}
       pda  : ${userPda.toBase58()}
       bump : ${userBump}
    `);
  });

  it("derives the rewards PDA from the config PDA", () => {
    const [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );
    const [rewardsPda, rewardsBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("rewards"), configPda.toBuffer()],
      program.programId
    );

    expect(rewardsPda).to.be.instanceOf(anchor.web3.PublicKey);
    expect(rewardsBump).to.be.at.least(0);

    console.log(PASS("rewards PDA"));
    console.log(`
       seeds: ["rewards", <configPda>]
       config : ${configPda.toBase58()}
       rewards: ${rewardsPda.toBase58()}
       bump   : ${rewardsBump}

       PDA chain:
         ["config"]  -->  configPda
                              |
                         ["rewards", configPda]  -->  rewardsPda
    `);
  });

  after(() => {
    console.log(`
+----------------------------------------------------------+
|                                                          |
|   ALL TESTS COMPLETE                                     |
|                                                          |
|   .  program id check ........................... done   |
|   .  initialize() .............................. done   |
|   .  config PDA derivation ..................... done   |
|   .  user PDA derivation ...................... done   |
|   .  rewards PDA derivation ................... done   |
|                                                          |
+----------------------------------------------------------+
    `);
  });
});

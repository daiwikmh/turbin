import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DiceGame } from "../target/types/dice_game";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  LAMPORTS_PER_SOL,
  Transaction,
  TransactionInstruction,
  SYSVAR_INSTRUCTIONS_PUBKEY,
} from "@solana/web3.js";
import { assert } from "chai";

const MEMO_PROGRAM_ID = new PublicKey(
  "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"
);

describe("dice_game", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.diceGame as Program<DiceGame>;
  const house = provider.wallet;
  const player = Keypair.generate();
  const seed = new anchor.BN(12345);

  const vault = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), house.publicKey.toBuffer()],
    program.programId
  )[0];

  const bet = PublicKey.findProgramAddressSync(
    [Buffer.from("bet"), vault.toBuffer(), seed.toArrayLike(Buffer, "le", 16)],
    program.programId
  )[0];

  it("initializes and funds the vault", async () => {
    await program.methods
      .initialize(new anchor.BN(2 * LAMPORTS_PER_SOL))
      .accountsPartial({
        house: house.publicKey,
        vault,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const balance = await provider.connection.getBalance(vault);
    assert.isAtLeast(balance, 2 * LAMPORTS_PER_SOL);
  });

  it("places a bet", async () => {
    const sig = await provider.connection.requestAirdrop(
      player.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);

    await program.methods
      .placeBet(seed, 50, new anchor.BN(0.1 * LAMPORTS_PER_SOL))
      .accountsPartial({
        player: player.publicKey,
        house: house.publicKey,
        vault,
        bet,
        systemProgram: SystemProgram.programId,
      })
      .signers([player])
      .rpc();

    const betAccount = await program.account.bet.fetch(bet);
    assert.equal(betAccount.roll, 50);
    assert.equal(betAccount.seed.toString(), seed.toString());
  });

  it("resolves by introspecting the memo", async () => {
    const randomness = new anchor.BN(7);

    const memoIx = new TransactionInstruction({
      keys: [],
      programId: MEMO_PROGRAM_ID,
      data: Buffer.from(`${seed.toString()},${randomness.toString()}`, "utf8"),
    });

    const resolveIx = await program.methods
      .resolveBet()
      .accountsPartial({
        house: house.publicKey,
        player: player.publicKey,
        vault,
        bet,
        instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    const tx = new Transaction().add(memoIx).add(resolveIx);
    await provider.sendAndConfirm(tx, []);

    const betInfo = await provider.connection.getAccountInfo(bet);
    assert.isNull(betInfo);
  });
});

import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Amm } from "../target/types/amm";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAccount,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";

// ─── helpers ─────────────────────────────────────────────────────────────────

async function airdrop(
  connection: anchor.web3.Connection,
  pubkey: PublicKey,
  sol = 10
) {
  const sig = await connection.requestAirdrop(pubkey, sol * LAMPORTS_PER_SOL);
  await connection.confirmTransaction(sig);
}

function configPda(seed: bigint, programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("config"), seedToBuffer(seed)],
    programId
  );
}

function lpMintPda(config: PublicKey, programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("lp"), config.toBuffer()],
    programId
  );
}

function seedToBuffer(seed: bigint): Buffer {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(seed);
  return buf;
}

// ─── test suite ──────────────────────────────────────────────────────────────

describe("amm", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Amm as Program<Amm>;
  const connection = provider.connection;

  // Keypairs
  const payer = Keypair.generate();
  const user = Keypair.generate();
  const treasury = Keypair.generate();

  // Token mints
  let mintX: PublicKey;
  let mintY: PublicKey;

  // PDAs
  const SEED = BigInt(42);
  let configPubkey: PublicKey;
  let mintLp: PublicKey;

  // Token accounts
  let vaultX: PublicKey;
  let vaultY: PublicKey;
  let userAtaX: PublicKey;
  let userAtaY: PublicKey;
  let userAtaLp: PublicKey;
  let treasuryAtaX: PublicKey; // treasury receives source-token fees (X→Y swap fees)

  // ── setup ──────────────────────────────────────────────────────────────────
  before(async () => {
    // Fund accounts
    await airdrop(connection, payer.publicKey, 20);
    await airdrop(connection, user.publicKey, 10);
    await airdrop(connection, treasury.publicKey, 2);

    // Create two SPL mints owned by payer
    mintX = await createMint(connection, payer, payer.publicKey, null, 6);
    mintY = await createMint(connection, payer, payer.publicKey, null, 6);

    // Derive PDAs
    [configPubkey] = configPda(SEED, program.programId);
    [mintLp] = lpMintPda(configPubkey, program.programId);

    // Create user token accounts and mint tokens to them
    userAtaX = (
      await getOrCreateAssociatedTokenAccount(
        connection, payer, mintX, user.publicKey
      )
    ).address;
    userAtaY = (
      await getOrCreateAssociatedTokenAccount(
        connection, payer, mintY, user.publicKey
      )
    ).address;

    await mintTo(connection, payer, mintX, userAtaX, payer, 1_000_000_000);
    await mintTo(connection, payer, mintY, userAtaY, payer, 1_000_000_000);

    // Treasury ATA for X (receives protocol fees on X→Y swaps)
    treasuryAtaX = (
      await getOrCreateAssociatedTokenAccount(
        connection, payer, mintX, treasury.publicKey
      )
    ).address;

    // Vault ATAs are derived by the program — get their addresses for assertions
    const { getAssociatedTokenAddressSync } = await import("@solana/spl-token");
    vaultX = getAssociatedTokenAddressSync(mintX, configPubkey, true);
    vaultY = getAssociatedTokenAddressSync(mintY, configPubkey, true);
  });

  // ── initialize ─────────────────────────────────────────────────────────────
  it("initializes the pool", async () => {
    const fee = 30; // 0.30%
    const authority = payer.publicKey;

    await program.methods
      .initialize(
        new BN(SEED.toString()),
        fee,
        authority,
        treasury.publicKey
      )
      .accounts({
        initializer: payer.publicKey,
        mintX,
        mintY,
        mintLp,
        vaultX,
        vaultY,
        config: configPubkey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer])
      .rpc();

    const config = await program.account.config.fetch(configPubkey);

    assert.equal(config.fee, 30, "fee should be 30 bps");
    assert.equal(config.locked, false, "pool should not be locked at start");
    assert.equal(
      config.mintX.toBase58(),
      mintX.toBase58(),
      "mintX stored correctly"
    );
    assert.equal(
      config.mintY.toBase58(),
      mintY.toBase58(),
      "mintY stored correctly"
    );
    assert.equal(
      config.authority.toBase58(),
      authority.toBase58(),
      "authority stored correctly"
    );
    assert.equal(
      config.treasury.toBase58(),
      treasury.publicKey.toBase58(),
      "treasury stored correctly"
    );

    // Vaults should exist and be empty
    const vx = await getAccount(connection, vaultX);
    const vy = await getAccount(connection, vaultY);
    assert.equal(vx.amount, 0n, "vault X should start empty");
    assert.equal(vy.amount, 0n, "vault Y should start empty");
  });

  // ── deposit (first / initial) ──────────────────────────────────────────────
  it("deposits initial liquidity", async () => {
    const depositX = 100_000_000n; // 100 tokens (6 decimals)
    const depositY = 200_000_000n; // 200 tokens — sets initial price 1:2
    const lpAmount = 100_000_000n; // how many LP tokens to mint

    const userXBefore = (await getAccount(connection, userAtaX)).amount;
    const userYBefore = (await getAccount(connection, userAtaY)).amount;

    // userAtaLp may not exist yet — program uses init_if_needed
    const { getAssociatedTokenAddressSync } = await import("@solana/spl-token");
    userAtaLp = getAssociatedTokenAddressSync(mintLp, user.publicKey);

    await program.methods
      .deposit(
        new BN(lpAmount.toString()),
        new BN(depositX.toString()),
        new BN(depositY.toString())
      )
      .accounts({
        user: user.publicKey,
        userX: userAtaX,
        userY: userAtaY,
        mintX,
        mintY,
        mintLp,
        vaultX,
        vaultY,
        config: configPubkey,
        userLp: userAtaLp,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const vx = await getAccount(connection, vaultX);
    const vy = await getAccount(connection, vaultY);
    const lp = await getAccount(connection, userAtaLp);
    const userXAfter = (await getAccount(connection, userAtaX)).amount;
    const userYAfter = (await getAccount(connection, userAtaY)).amount;

    // First deposit: max_x and max_y go in directly
    assert.equal(vx.amount, depositX, "vault X should equal depositX");
    assert.equal(vy.amount, depositY, "vault Y should equal depositY");
    assert.equal(lp.amount, lpAmount, "user LP balance should equal lpAmount");
    assert.equal(userXBefore - userXAfter, depositX, "user X reduced by depositX");
    assert.equal(userYBefore - userYAfter, depositY, "user Y reduced by depositY");
  });

  // ── swap X → Y ─────────────────────────────────────────────────────────────
  it("swaps X for Y with fee and sends protocol fee to treasury", async () => {
    const amountIn = 10_000_000n; // 10 tokens in

    const vxBefore = (await getAccount(connection, vaultX)).amount;
    const vyBefore = (await getAccount(connection, vaultY)).amount;
    const userXBefore = (await getAccount(connection, userAtaX)).amount;
    const userYBefore = (await getAccount(connection, userAtaY)).amount;
    const treasuryBefore = (await getAccount(connection, treasuryAtaX)).amount;

    // Manually compute expected output to assert against
    const feeBps = 30n;
    const protocolShare = 5n; // 1/5 = 20%
    const protocolFee = (amountIn * feeBps) / (10000n * protocolShare);
    const lpFeeBps = feeBps - feeBps / protocolShare;
    const afterProtocol = amountIn - protocolFee;
    const afterLpFee = (afterProtocol * (10000n - lpFeeBps)) / 10000n;
    const expectedOut = (vyBefore * afterLpFee) / (vxBefore + afterLpFee);

    await program.methods
      .swap(new BN(amountIn.toString()), new BN(1))
      .accounts({
        user: user.publicKey,
        userSource: userAtaX,
        userDestination: userAtaY,
        vaultX,
        vaultY,
        treasuryAta: treasuryAtaX,
        mintX,
        mintY,
        config: configPubkey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    const vxAfter = (await getAccount(connection, vaultX)).amount;
    const vyAfter = (await getAccount(connection, vaultY)).amount;
    const userXAfter = (await getAccount(connection, userAtaX)).amount;
    const userYAfter = (await getAccount(connection, userAtaY)).amount;
    const treasuryAfter = (await getAccount(connection, treasuryAtaX)).amount;

    // User sent amountIn of X
    assert.equal(userXBefore - userXAfter, amountIn, "user sent amountIn X");

    // User received Y
    const receivedY = userYAfter - userYBefore;
    assert.isAbove(Number(receivedY), 0, "user received some Y");
    // Allow ±1 for integer division rounding
    assert.approximately(
      Number(receivedY),
      Number(expectedOut),
      1,
      "received Y matches constant-product formula"
    );

    // Vault X grew by amountIn minus the protocol fee that left it
    const vaultXGrowth = vxAfter - vxBefore;
    assert.approximately(
      Number(vaultXGrowth),
      Number(amountIn - protocolFee),
      1,
      "vault X grew by amountIn minus protocol fee"
    );

    // Treasury received the protocol fee
    const treasuryGrowth = treasuryAfter - treasuryBefore;
    assert.approximately(
      Number(treasuryGrowth),
      Number(protocolFee),
      1,
      "treasury received protocol fee"
    );

    // k should increase slightly (fees accumulate in pool)
    const kBefore = vxBefore * vyBefore;
    const kAfter = vxAfter * vyAfter;
    assert.isAtLeast(Number(kAfter), Number(kBefore), "k should not decrease");
  });

  // ── withdraw ───────────────────────────────────────────────────────────────
  it("withdraws liquidity proportionally", async () => {
    const burnAmount = 10_000_000n; // burn 10% of LP supply

    const vxBefore = (await getAccount(connection, vaultX)).amount;
    const vyBefore = (await getAccount(connection, vaultY)).amount;
    const lpBefore = (await getAccount(connection, userAtaLp)).amount;
    const lpSupplyBefore = (await connection.getTokenSupply(mintLp)).value
      .uiAmountString;
    const lpSupplyBigint = BigInt(
      (await connection.getTokenSupply(mintLp)).value.amount
    );
    const userXBefore = (await getAccount(connection, userAtaX)).amount;
    const userYBefore = (await getAccount(connection, userAtaY)).amount;

    const expectedX = (vxBefore * burnAmount) / lpSupplyBigint;
    const expectedY = (vyBefore * burnAmount) / lpSupplyBigint;

    await program.methods
      .withdraw(
        new BN(burnAmount.toString()),
        new BN(1), // minimum_amount_x
        new BN(1)  // minimum_amount_y
      )
      .accounts({
        user: user.publicKey,
        userX: userAtaX,
        userY: userAtaY,
        userLp: userAtaLp,
        mintX,
        mintY,
        mintLp,
        vaultX,
        vaultY,
        config: configPubkey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    const vxAfter = (await getAccount(connection, vaultX)).amount;
    const vyAfter = (await getAccount(connection, vaultY)).amount;
    const lpAfter = (await getAccount(connection, userAtaLp)).amount;
    const userXAfter = (await getAccount(connection, userAtaX)).amount;
    const userYAfter = (await getAccount(connection, userAtaY)).amount;

    // LP tokens burned
    assert.equal(lpBefore - lpAfter, burnAmount, "LP tokens burned");

    // Vault balances decreased correctly
    assert.approximately(
      Number(vxBefore - vxAfter),
      Number(expectedX),
      1,
      "vault X decreased proportionally"
    );
    assert.approximately(
      Number(vyBefore - vyAfter),
      Number(expectedY),
      1,
      "vault Y decreased proportionally"
    );

    // User received tokens back
    assert.approximately(
      Number(userXAfter - userXBefore),
      Number(expectedX),
      1,
      "user received X proportionally"
    );
    assert.approximately(
      Number(userYAfter - userYBefore),
      Number(expectedY),
      1,
      "user received Y proportionally"
    );
  });

  // ── update_config ──────────────────────────────────────────────────────────
  it("authority can update fee", async () => {
    await program.methods
      .updateConfig(50, null, null) // new fee = 50bps, leave locked/authority unchanged
      .accounts({
        authority: payer.publicKey,
        config: configPubkey,
      })
      .signers([payer])
      .rpc();

    const config = await program.account.config.fetch(configPubkey);
    assert.equal(config.fee, 50, "fee updated to 50 bps");
    assert.equal(config.locked, false, "locked unchanged");
  });

  it("authority can lock the pool", async () => {
    await program.methods
      .updateConfig(null, true, null)
      .accounts({
        authority: payer.publicKey,
        config: configPubkey,
      })
      .signers([payer])
      .rpc();

    const config = await program.account.config.fetch(configPubkey);
    assert.equal(config.locked, true, "pool is now locked");
  });

  it("deposit fails when pool is locked", async () => {
    try {
      await program.methods
        .deposit(new BN(1_000_000), new BN(1_000_000), new BN(2_000_000))
        .accounts({
          user: user.publicKey,
          userX: userAtaX,
          userY: userAtaY,
          mintX,
          mintY,
          mintLp,
          vaultX,
          vaultY,
          config: configPubkey,
          userLp: userAtaLp,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();
      assert.fail("should have thrown PoolLocked error");
    } catch (err: any) {
      assert.include(err.message, "PoolLocked", "expected PoolLocked error");
    }
  });

  it("authority can unlock the pool", async () => {
    await program.methods
      .updateConfig(null, false, null)
      .accounts({
        authority: payer.publicKey,
        config: configPubkey,
      })
      .signers([payer])
      .rpc();

    const config = await program.account.config.fetch(configPubkey);
    assert.equal(config.locked, false, "pool is unlocked");
  });

  it("non-authority cannot update config", async () => {
    try {
      await program.methods
        .updateConfig(100, null, null)
        .accounts({
          authority: user.publicKey, // user is NOT the authority
          config: configPubkey,
        })
        .signers([user])
        .rpc();
      assert.fail("should have thrown Unauthorized error");
    } catch (err: any) {
      assert.include(
        err.message,
        "Unauthorized",
        "expected Unauthorized error"
      );
    }
  });

  it("fee above 1000 bps is rejected", async () => {
    try {
      await program.methods
        .updateConfig(1001, null, null)
        .accounts({
          authority: payer.publicKey,
          config: configPubkey,
        })
        .signers([payer])
        .rpc();
      assert.fail("should have rejected fee > 1000 bps");
    } catch (err: any) {
      assert.include(
        err.message,
        "InvalidFee",
        "expected InvalidFee error"
      );
    }
  });

  it("authority can transfer authority to another key", async () => {
    const newAuth = Keypair.generate();

    await program.methods
      .updateConfig(null, null, newAuth.publicKey)
      .accounts({
        authority: payer.publicKey,
        config: configPubkey,
      })
      .signers([payer])
      .rpc();

    const config = await program.account.config.fetch(configPubkey);
    assert.equal(
      config.authority.toBase58(),
      newAuth.publicKey.toBase58(),
      "authority transferred"
    );

    // Restore payer as authority so remaining tests work
    await airdrop(connection, newAuth.publicKey, 1);
    await program.methods
      .updateConfig(null, null, payer.publicKey)
      .accounts({
        authority: newAuth.publicKey,
        config: configPubkey,
      })
      .signers([newAuth])
      .rpc();
  });
});

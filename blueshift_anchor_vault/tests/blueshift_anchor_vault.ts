import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { BlueshiftAnchorVault } from "../target/types/blueshift_anchor_vault";

describe("blueshift_anchor_vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .blueshiftAnchorVault as Program<BlueshiftAnchorVault>;

  const signer = provider.wallet as anchor.Wallet;

  const [vault] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), signer.publicKey.toBuffer()],
    program.programId
  );

  let minimumRent: number;

  before(async () => {
    minimumRent =
      await provider.connection.getMinimumBalanceForRentExemption(0);
  });

  async function signerBalance() {
    return provider.connection.getBalance(signer.publicKey);
  }

  async function vaultBalance() {
    return provider.connection.getBalance(vault);
  }

  async function expectAnchorError(
    promise: Promise<unknown>,
    expectedCode: string
  ) {
    try {
      await promise;
      expect.fail(`expected Anchor error ${expectedCode}`);
    } catch (error) {
      const anchorError = error as anchor.AnchorError;
      expect(anchorError.error.errorCode.code).to.equal(expectedCode);
    }
  }

  it("deposits lamports into the vault PDA", async () => {
    const amount = minimumRent + 1_000_000;

    const beforeSigner = await signerBalance();
    const beforeVault = await vaultBalance();

    expect(beforeVault).to.equal(0);

    await program.methods
      .deposit(new anchor.BN(amount))
      .accounts({
        signer: signer.publicKey,
      })
      .rpc();

    const afterVault = await vaultBalance();
    const afterSigner = await signerBalance();

    expect(afterVault).to.equal(amount);
    expect(beforeSigner - afterSigner).to.be.greaterThanOrEqual(amount);
  });

  it("rejects a second deposit while the vault still holds lamports", async () => {
    await expectAnchorError(
      program.methods
        .deposit(new anchor.BN(minimumRent + 1_000_000))
        .accounts({
          signer: signer.publicKey,
        })
        .rpc(),
      "VaultAlreadyExists"
    );
  });

  it("withdraws all lamports from the vault PDA back to the signer", async () => {
    const beforeSigner = await signerBalance();
    const beforeVault = await vaultBalance();

    expect(beforeVault).to.be.greaterThan(0);

    await program.methods
      .withdraw()
      .accounts({
        signer: signer.publicKey,
      })
      .rpc();

    const afterVault = await vaultBalance();
    const afterSigner = await signerBalance();

    expect(afterVault).to.equal(0);
    expect(afterSigner).to.be.greaterThan(beforeSigner);
  });

  it("rejects deposits that do not exceed the rent-exempt minimum", async () => {
    await expectAnchorError(
      program.methods
        .deposit(new anchor.BN(minimumRent))
        .accounts({
          signer: signer.publicKey,
        })
        .rpc(),
      "InvalidAmount"
    );
  });

  it("rejects withdraw when the vault is empty", async () => {
    await expectAnchorError(
      program.methods
        .withdraw()
        .accounts({
          signer: signer.publicKey,
        })
        .rpc(),
      "InvalidAmount"
    );
  });
});

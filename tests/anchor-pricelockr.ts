import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorPricelockr } from "../target/types/anchor_pricelockr";
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
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { expect } from "chai";

describe("anchor-pricelockr", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace
    .anchorPricelockr as Program<AnchorPricelockr>;
  let mint: anchor.web3.PublicKey;
  let userAta: anchor.web3.PublicKey;
  let vaultAta: anchor.web3.PublicKey;
  let winnerAta: anchor.web3.PublicKey;
  let contestantsKps: Keypair[] = [];
  const user = provider.wallet as anchor.Wallet;
  let vault: anchor.web3.PublicKey;
  let tournament: anchor.web3.PublicKey;
  let winner: anchor.web3.PublicKey;
  let amount = new anchor.BN(1_000_000_000);
  it("Is initialized!", async () => {
    mint = await createMint(
      provider.connection,
      user.payer,
      user.publicKey,
      null,
      6
    );
    [vault] = await PublicKey.findProgramAddress(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      program.programId
    );
    [tournament] = await PublicKey.findProgramAddress(
      [Buffer.from("nft"), user.publicKey.toBuffer()],
      program.programId
    );
    // User ATA
    userAta = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        user.payer,
        mint,
        user.publicKey
      )
    ).address;
    vaultAta = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        user.payer,
        mint,
        vault,
        true
      )
    ).address;
    const preUserBalance = await provider.connection.getTokenAccountBalance(
      userAta
    );
    await mintTo(
      provider.connection,
      user.payer,
      mint,
      userAta,
      user.publicKey,
      1_000_000_000
    );

    const tx = await program.methods
      .initialize(amount)
      .accounts({
        vault: vault,
        tournament: tournament,
        user: user.publicKey,
        creator: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("Your transaction signature", tx);
    const postUserBalance = await provider.connection.getTokenAccountBalance(
      userAta
    );
    console.log(
      "User balance(pre -> post): ",
      Number(preUserBalance.value.amount) / 1_000_000_000 + " SOL",
      "->",
      Number(postUserBalance.value.amount) / 1_000_000_000 + " SOL"
    );
  });
  it("Enroll Users", async () => {
    const randomPubkeys: PublicKey[] = [];
    contestantsKps = [];
    for (let i = 0; i < 10; i++) {
      const keyPair = Keypair.generate();
      contestantsKps.push(keyPair);
      randomPubkeys.push(keyPair.publicKey);
    }
    const tx = await program.methods
      .addContestants(randomPubkeys)
      .accounts({
        creator: user.publicKey,
        tournament: tournament,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    const contestants = await program.account.tournament.fetch(tournament);
    console.log("Contestants Enrolled: ", contestants.contestants);
  });
  it("Result Announcement", async () => {
    const tx = await program.methods
      .tournamentResult()
      .accounts({
        vault: vault,
        vaultAta: vaultAta,
        user: user.publicKey,
        userAta: userAta,
        tournament: tournament,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    const tournamentWinner = await program.account.tournament.fetch(tournament);
    console.log("Winner: ", tournamentWinner.winner);
  });
  it("Claim Reward", async () => {
    // Fetch on-chain winner and find matching keypair
    const t = await program.account.tournament.fetch(tournament);
    const winnerWallet = t.winner as PublicKey;
    const winnerKp = contestantsKps.find((kp) =>
      kp.publicKey.equals(winnerWallet)
    );
    if (!winnerKp) throw new Error("Winner keypair not found in contestants");

    [winner] = await PublicKey.findProgramAddress(
      [Buffer.from("win"), winnerWallet.toBuffer()],
      program.programId
    );

    await provider.connection.requestAirdrop(winnerWallet, LAMPORTS_PER_SOL);

    winnerAta = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        user.payer,
        mint,
        winnerWallet
      )
    ).address;

    const tx = await program.methods
      .claimReward()
      .accounts({
        vault: vault,
        vaultAta: vaultAta,
        user: winnerWallet,
        userAta: userAta,
        winner: winner,
        winnerAta: winnerAta,
        tournament: tournament,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([winnerKp])
      .rpc();
    const userBalance = await provider.connection.getTokenAccountBalance(
      userAta
    );
    const winnerBalance = await provider.connection.getTokenAccountBalance(
      winnerAta
    );
    const vaultBalance = await provider.connection.getTokenAccountBalance(
      vaultAta
    );
    console.log("User Balance: ", userBalance.value.amount);
    console.log("Winner Balance: ", winnerBalance.value.amount);
    console.log("Vault Balance: ", vaultBalance.value.amount);

    expect(Number(userBalance.value.amount)).to.equal(0);
    expect(Number(winnerBalance.value.amount)).to.equal(1_000_000_000);
    expect(Number(vaultBalance.value.amount)).to.equal(0);
    expect(
      (await program.account.tournament.fetch(tournament)).priceClaimed
    ).to.equal(true);
  });
});

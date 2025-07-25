import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolStake } from "../target/types/sol_stake";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo } from '@solana/spl-token';
import type { Account as TokenAccount } from "@solana/spl-token";

describe("sol-stake", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const program = anchor.workspace.solStake as Program<SolStake>;

  let admin: anchor.web3.Keypair;
  let user: anchor.web3.Keypair;
  let stake_token_mint: anchor.web3.PublicKey;
  let reward_token_mint: anchor.web3.PublicKey;
  let stake_pool: anchor.web3.PublicKey;
  let reward_vault: anchor.web3.PublicKey;
  let stake_vault: anchor.web3.PublicKey;
  let pool_authority: anchor.web3.PublicKey;
  let user_stake: anchor.web3.PublicKey;
  let user_stake_token_account: TokenAccount;
  let user_reward_token_account: TokenAccount;

  before(async () => {
    admin = anchor.web3.Keypair.generate();
    user = anchor.web3.Keypair.generate();

    const sig = await anchor.getProvider().connection.requestAirdrop(
      admin.publicKey, 1e9
    );
    await anchor.getProvider().connection.confirmTransaction(sig);
    const userSign = await anchor.getProvider().connection.requestAirdrop(
      user.publicKey,
      1e9
    );
    await anchor.getProvider().connection.confirmTransaction(userSign);

    stake_token_mint = await createMint(
      provider.connection,
      admin,
      admin.publicKey,
      null,
      9
    );

    reward_token_mint = await createMint(
      provider.connection,
      admin,
      admin.publicKey,
      null,
      9
    );

    user_stake_token_account = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      user,
      stake_token_mint,
      user.publicKey
    );

    await mintTo(
      provider.connection,
      admin,
      stake_token_mint,
      user_stake_token_account.address,
      admin,
      BigInt(100_000_000_000)
    );

    user_reward_token_account = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      user,
      reward_token_mint,
      user.publicKey
    );

    [stake_pool] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("stake_pool"), admin.publicKey.toBuffer(), stake_token_mint.toBuffer()],
      program.programId
    );

    [user_stake] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_stake"), stake_pool.toBuffer(), user.publicKey.toBuffer()],
      program.programId
    );

    [reward_vault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("reward_vault"), stake_pool.toBuffer()],
      program.programId
    );

    [stake_vault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("stake_vault"), stake_pool.toBuffer()],
      program.programId
    );

    [pool_authority] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("pool_authority"), stake_pool.toBuffer()],
      program.programId
    );
  });

  it("Is initialized!", async () => {
    const tx = await program.methods
      .initialize(new anchor.BN(1000000))
      .accounts({
        user: admin.publicKey,
        pool: stake_pool,
        stakeTokenMint: stake_token_mint,
        rewardTokenMint: reward_token_mint,
        rewardTokenVault: reward_vault,
        stakeTokenVault: stake_vault,
        authority: pool_authority,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();

    console.log("Your transaction signature", tx);
  });

  it("Can stake tokens!", async () => {
    const tx = await program.methods
      .stake(new anchor.BN(100))
      .accounts({
        user: user.publicKey,
        pool: stake_pool,
        userStakeAccount: user_stake_token_account.address,
        userRewardAccount: user_reward_token_account.address,
        stakeTokenVault: stake_vault,
        rewardTokenVault: reward_vault,
        userStake: user_stake,
        authority: pool_authority,
      })
      .signers([user])
      .rpc();

    console.log("Your transaction signature", tx);
  });
});

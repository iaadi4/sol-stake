import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolStake } from "../target/types/sol_stake";
import { createMint } from '@solana/spl-token';

describe("sol-stake", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const program = anchor.workspace.solStake as Program<SolStake>;

  let user: anchor.web3.Keypair;
  let stake_token_mint: anchor.web3.PublicKey;
  let reward_token_mint: anchor.web3.PublicKey;
  let stake_pool: anchor.web3.PublicKey;
  let reward_vault: anchor.web3.PublicKey;
  let stake_vault: anchor.web3.PublicKey;
  let pool_authority: anchor.web3.PublicKey;

  before(async () => {
    user = anchor.web3.Keypair.generate();
    const sig = await provider.connection.requestAirdrop(user.publicKey, 1e9);
    await provider.connection.confirmTransaction(sig);

    stake_token_mint = await createMint(
      provider.connection,
      user,
      user.publicKey,
      null,
      9
    );

    reward_token_mint = await createMint(
      provider.connection,
      user,
      user.publicKey,
      null,
      9
    );

    [stake_pool] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("stake_pool"), user.publicKey.toBuffer(), stake_token_mint.toBuffer()],
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
        user: user.publicKey,
        stakePool: stake_pool,
        stakeTokenMint: stake_token_mint,
        rewardTokenMint: reward_token_mint,
        rewardVault: reward_vault,
        stakeVault: stake_vault,
        poolAuthority: pool_authority,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    console.log("Your transaction signature", tx);
  });
});

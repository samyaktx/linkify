import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Linkify } from "../target/types/linkify";
import dotenv from "dotenv";
import { expect } from "chai";
dotenv.config();

const confirmTx = async (signature: string) => {
  const latestBlockhash = await anchor
    .getProvider()
    .connection
    .getLatestBlockhash();

    await anchor.getProvider().connection.confirmTransaction({
      signature,
      ...latestBlockhash,
    }, "confirmed");

    return signature;
};

describe("linkify", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.Linkify as Program<Linkify>;

  it("Create user", async () => {
    // Add your test here.

    const user_prvt = JSON.parse(process.env.PRIVATE_KEY_USER1!);
    const user_1 = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(user_prvt));
    const user_1_name = "krsnax";
      
    const [userAccount, _bump] = anchor.web3.PublicKey.findProgramAddressSync([
        Buffer.from("user"), 
        user_1.publicKey.toBuffer(),
      ],
      program.programId
    );
    
    console.log("Derived User Account:", userAccount.toBase58());
    console.log("Program ID:", program.programId.toBase58());
    console.log("User Public Key:", user_1.publicKey.toBase58());

    const tx = await program.methods
      .createUser(user_1_name)
      .accounts({
        // @ts-ignore
        // getting error: Object literal may only specify known properties, and 'user' does not exist in type
        // Error: AnchorError caused by account: user. Error Code: AccountDidNotDeserialize. Error Number: 3003. Error Message: Failed to deserialize the account.
        user: userAccount,
        signer: user_1.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user_1])
      .rpc()
      .then(confirmTx); 

    console.log("Transaction signature:", tx);

    const account = await program.account.userInfo.fetch(userAccount);

    expect(account.name).to.equal(user_1_name);
    expect(account.userPubkey.equals(user_1.publicKey)).to.be.true;
    expect(account.reqSentCount).to.equal(0);
    expect(account.reqReceivedCount).to.equal(0);
  });
});

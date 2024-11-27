import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Linkify } from "../target/types/linkify";
import dotenv from "dotenv";
import { expect } from "chai";
import { BN } from "bn.js";
dotenv.config();

anchor.setProvider(anchor.AnchorProvider.env());
const program = anchor.workspace.Linkify as Program<Linkify>;

let requesterOne: anchor.web3.Keypair;
let requesterTwo: anchor.web3.Keypair;
let acceptor: anchor.web3.Keypair;

describe("linkify", () => {
  before(async () => {
    let airdropAmount = (anchor.web3.LAMPORTS_PER_SOL / 10) * 7;
    const userOnePrvtAcceptor = JSON.parse(process.env.PRIVATE_KEY_USER1!);
    const userTwoPrvtRequester = JSON.parse(process.env.PRIVATE_KEY_USER2!);
    const userPrvtAcceptor = JSON.parse(process.env.PRIVATE_KEY_USER3!);

    requesterOne = anchor.web3.Keypair.fromSecretKey(
      Uint8Array.from(userOnePrvtAcceptor)
    );
    requesterTwo = anchor.web3.Keypair.fromSecretKey(
      Uint8Array.from(userTwoPrvtRequester)
    );
    acceptor = anchor.web3.Keypair.fromSecretKey(
      Uint8Array.from(userPrvtAcceptor)
    );

    await requestAirdrop(requesterOne.publicKey, airdropAmount);
    await requestAirdrop(requesterTwo.publicKey, airdropAmount);
    await requestAirdrop(acceptor.publicKey, airdropAmount);

    await createUser(requesterOne, "requesterOne");
    await createUser(requesterTwo, "requesterTwo");
    await createUser(acceptor, "acceptor");
  });

  const confirmTx = async (signature: string) => {
    const latestBlockhash = await anchor
      .getProvider()
      .connection.getLatestBlockhash();

    await anchor.getProvider().connection.confirmTransaction({
        signature,
        ...latestBlockhash,
      },
      "confirmed"
    );

    return signature;
  };

  const requestAirdrop = async (
    publicKey: anchor.web3.PublicKey,
    amount: number
  ) => {
    try {
      const user_balance = await anchor
        .getProvider()
        .connection.getBalance(publicKey);

      if (user_balance <= anchor.web3.LAMPORTS_PER_SOL / 2) {
        const signature = await anchor
          .getProvider()
          .connection
          .requestAirdrop(publicKey, amount);
        return await confirmTx(signature);
      } else {
        console.log(`User ${publicKey}: doesn't required airdrop`);
      }
    } catch (error) {
      console.error("Error requesting airdrop:", error);
      throw error;
    }
  };

  const createUser = async (user: anchor.web3.Keypair, username: string) => {
    const [userAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user"), user.publicKey.toBuffer()],
      program.programId
    );

    let userAccountInfo = await program.provider.connection.getAccountInfo(
      userAccount
    );
    if (userAccountInfo == null) {
      await program.methods
        .createUser(username)
        .accounts({
          //@ts-ignore
          user: userAccount,
          signer: user.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([user])
        .rpc()
        .then(confirmTx);
    } else {
      console.log(`User ${username}: already exists`);
    }
  };

  const getAccountAddress = async (user: anchor.web3.Keypair) => {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user"), user.publicKey.toBuffer()],
      program.programId
    )[0];
  };

  const logPDA = (seeds: Buffer[], programId: anchor.web3.PublicKey) => {
    const [pda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      seeds,
      programId
    );
    console.log(`PDA: ${pda.toBase58()}, Bump: ${bump}`);
  };

  it("UserOne requesting connection with Acceptor", async () => {
    const acceptorPubKey = acceptor.publicKey;
    const [connectionAccount, bump] =
      anchor.web3.PublicKey.findProgramAddressSync([
          Buffer.from("connect"),
          acceptor.publicKey.toBuffer(),
          new BN(0).toArrayLike(Buffer, "le", 4),
        ],
        program.programId
      );

    logPDA([
        Buffer.from("connect"),
        acceptor.publicKey.toBuffer(),
        new BN(0).toArrayLike(Buffer, "le", 4),
      ],
      program.programId
    );

    const tx = await program.methods
      .requestConnection(acceptorPubKey)
      .accountsPartial({
        connection: connectionAccount,
        signer: requesterOne.publicKey,
        requesterAcc: await getAccountAddress(requesterOne),
        acceptorAcc: await getAccountAddress(acceptor),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([requesterOne])
      .rpc()
      .then(confirmTx);

    console.log("Request connection transaction signature:", tx);

    const connection = await program.account.connection.fetch(
      connectionAccount
    );
    expect(connection.areConnected).to.be.false;
    expect(connection.requester.equals(requesterOne.publicKey)).to.be.true;
    expect(connection.acceptor.equals(acceptorPubKey)).to.be.true;
  });

  it("UserTwo requesting connection with Acceptor", async () => {
    const acceptorPubKey = acceptor.publicKey;
    const [connectionAccount, bump] =
      anchor.web3.PublicKey.findProgramAddressSync([
          Buffer.from("connect"),
          acceptor.publicKey.toBuffer(),
          new BN(1).toArrayLike(Buffer, "le", 4),
        ],
        program.programId
      );

    logPDA([
        Buffer.from("connect"),
        acceptor.publicKey.toBuffer(),
        new BN(1).toArrayLike(Buffer, "le", 4),
      ],
      program.programId
    );

    const tx = await program.methods
      .requestConnection(acceptorPubKey)
      .accountsPartial({
        connection: connectionAccount,
        signer: requesterTwo.publicKey,
        requesterAcc: await getAccountAddress(requesterTwo),
        acceptorAcc: await getAccountAddress(acceptor),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([requesterTwo])
      .rpc()
      .then(confirmTx);

    console.log("Request connection transaction signature:", tx);

    const connection = await program.account.connection.fetch(
      connectionAccount
    );
    expect(connection.areConnected).to.be.false;
    expect(connection.requester.equals(requesterTwo.publicKey)).to.be.true;
    expect(connection.acceptor.equals(acceptorPubKey)).to.be.true;
  });

  it("Accepting UserOne connection with UserOne", async () => {
    const requesterPubKey = requesterOne.publicKey;
    const [connectionAccount, bump] =
      anchor.web3.PublicKey.findProgramAddressSync([
          Buffer.from("connect"),
          acceptor.publicKey.toBuffer(),
          new BN(0).toArrayLike(Buffer, "le", 4),
        ],
        program.programId
      );

    logPDA([
        Buffer.from("connect"),
        acceptor.publicKey.toBuffer(),
        new BN(0).toArrayLike(Buffer, "le", 4),
      ],
      program.programId
    );

    const tx = await program.methods
      .acceptConnection(requesterPubKey)
      .accounts({
        //@ts-ignore
        connection: connectionAccount,
        signer: acceptor.publicKey,
        acceptorAcc: await getAccountAddress(acceptor),
        requesterAcc: await getAccountAddress(requesterOne),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([acceptor])
      .rpc()
      .then(confirmTx);

    console.log("Accept connection transaction signature:", tx);

    const connection = await program.account.connection.fetch(
      connectionAccount
    );
    expect(connection.areConnected).to.be.true;
  });

  it("Rejecting UserTwo connection", async () => {
    const [connectionAccount, bump] = anchor.web3.PublicKey.findProgramAddressSync([
        Buffer.from("connect"),
        acceptor.publicKey.toBuffer(),
        new BN(1).toArrayLike(Buffer, "le", 4),
      ],
      program.programId
    );

    logPDA([
        Buffer.from("connect"),
        acceptor.publicKey.toBuffer(),
        new BN(1).toArrayLike(Buffer, "le", 4),
      ],
      program.programId
    );

    const expectedRequesterAcc = await getAccountAddress(requesterTwo);
    console.log(
      "Expected requester_acc address:",
      expectedRequesterAcc.toBase58()
    );

    const tx = await program.methods
      .rejectConnection(acceptor.publicKey)
      .accounts({
        //@ts-ignore
        connection: connectionAccount,
        signer: acceptor.publicKey,
        denialistAcc: await getAccountAddress(acceptor),
        requesterAcc: expectedRequesterAcc,
        requesterPubkey: requesterTwo.publicKey,
      })
      .signers([acceptor])
      .rpc()
      .then(confirmTx);

    console.log("Reject connection transaction signature:", tx);

    // Verify that the connection account is closed
    try {
      await program.account.connection.fetch(connectionAccount);
      expect.fail("Connection account should be closed after rejection.");
    } catch (error) {
      expect(error.message).to.contain("Account does not exist or has no data");
    }
  });

  it("Withdrawing SOL stake from Acceptor", async () => {
    const [connectionAccount, bump] = anchor.web3.PublicKey.findProgramAddressSync([
          Buffer.from("connect"),
          acceptor.publicKey.toBuffer(),
          new BN(0).toArrayLike(Buffer, "le", 4),
        ],
        program.programId
      );

    const tx = await program.methods
      .withdrawStake(acceptor.publicKey)
      .accounts({
        //@ts-ignore
        connection: connectionAccount,
        acceptorAcc: await getAccountAddress(acceptor),
        requesterAcc: await getAccountAddress(requesterOne),
        signer: acceptor.publicKey,
        requesterPubkey: requesterOne.publicKey,
        acceptorPubkey: acceptor.publicKey,
      })
      .signers([acceptor])
      .rpc()
      .then(confirmTx);

    console.log("Withdraw stake transaction signature:", tx);

    // Verify that the connection account is closed
    try {
      await program.account.connection.fetch(connectionAccount);
      expect.fail(
        "Connection account should be closed after stake withdrawal."
      );
    } catch (error) {
      expect(error.message).to.contain("Account does not exist or has no data");
    }
  });
});

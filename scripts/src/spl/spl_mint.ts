import { findAssociatedTokenPda, getCreateAssociatedTokenInstruction, getCreateAssociatedTokenInstructionAsync, getMintToInstruction, TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";
import wallet from "../../devnet-wallet.json" with { type: "json" };
import * as types from "@solana/kit";
import { appendFile, writeFileSync } from "node:fs";


const rpc = types.createSolanaRpc("https://api.devnet.solana.com");

const rpcSubscription =types.createSolanaRpcSubscriptions("wss://api.devnet.solana.com");

const token_decimals=1_000_000n;

const mint=types.address("8tHrfThcwQxwYCgMWR1ubQXHdz4TK8D4yhQLJJF8KWfK");

(async ()=>{

    try {

    const signer = await types.createKeyPairSignerFromBytes(
        new Uint8Array(wallet)
    );

    const [ata] = await findAssociatedTokenPda({
        mint,
        owner:signer.address,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    console.log("Associated Token Account:", ata);

    const createAtaTx=await getCreateAssociatedTokenInstructionAsync({
       payer:signer,
       mint,
       owner:signer.address,
    });

    const mintToTx=getMintToInstruction({
        mint,
        token:ata,
        mintAuthority:signer,
        amount:1n*token_decimals,
    });

    const {value: latestBlockhash}= await rpc.getLatestBlockhash().send();


    const msg=types.createTransactionMessage({version:0});
    
        const msgWithPayer=types.setTransactionMessageFeePayerSigner(signer,msg);
    
        const msgWithLifetime= types.setTransactionMessageLifetimeUsingBlockhash(
            latestBlockhash,
            msgWithPayer
        )

    const txMessage= types.appendTransactionMessageInstructions(
        [
            createAtaTx,
            mintToTx,
        ],
        msgWithLifetime
    );

    const signedTx = await types.signTransactionMessageWithSigners(txMessage);

    types.assertIsTransactionWithBlockhashLifetime(signedTx);

    const signature= types.getSignatureFromTransaction(signedTx);

     const sendAndConfirm= types.sendAndConfirmTransactionFactory({
            rpc,
            rpcSubscriptions: rpcSubscription,
        });

        await sendAndConfirm(signedTx,{commitment:"confirmed"});

        console.log("Minted tokens to ATA. Signature:", signature);

        

    }
    catch(error){
        console.error(error);
    }

})();


















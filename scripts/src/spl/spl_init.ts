import wallet from "../../devnet-wallet.json" with { type: "json" };
import * as types from "@solana/kit";
import {createSolanaRpcSubscriptions, assertIsTransactionWithBlockhashLifetime} from "@solana/kit";
import { getInitializeMintInstruction, getMintSize, TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";
import { getCreateAccountInstruction } from "@solana-program/system";


const rpc = types.createSolanaRpc("https://api.devnet.solana.com");

const rpcSubscription =createSolanaRpcSubscriptions("wss://api.devnet.solana.com");

(async ()=> {
try{
    const signer=await types.createKeyPairSignerFromBytes(
        new Uint8Array(wallet)
    )

    const mint = await types.generateKeyPairSigner();

    const space=BigInt(getMintSize());

    const rent = await rpc.getMinimumBalanceForRentExemption(space).send();

    const {value: latestBlockhash}=await rpc.getLatestBlockhash().send();

    const balance = await rpc.getBalance(signer.address).send();
    console.log("Balance:", balance.value);

    console.log(signer.address);

    const sendAndConfirm= types.sendAndConfirmTransactionFactory({
        rpc,
        rpcSubscriptions: rpcSubscription,
    });

    const msg=types.createTransactionMessage({version:0});

    const msgWithPayer=types.setTransactionMessageFeePayerSigner(signer,msg);

    const msgWithLifetime= types.setTransactionMessageLifetimeUsingBlockhash(
        latestBlockhash,
        msgWithPayer
    )
    

    const txMessag= types.appendTransactionMessageInstructions(
        [
        getCreateAccountInstruction({
            payer:signer,
            newAccount:mint,
            lamports:rent,
            space,
            programAddress:TOKEN_PROGRAM_ADDRESS,
        }),

        getInitializeMintInstruction({
            mint:mint.address,
            decimals:6,
            mintAuthority:signer.address,
        }),
        
    ],
    msgWithLifetime
    )
    const signedTx=await types.signTransactionMessageWithSigners(txMessag);
    
    assertIsTransactionWithBlockhashLifetime(signedTx);

    const signature=types.getSignatureFromTransaction(signedTx);
    console.log(signature)

    await sendAndConfirm(signedTx, {commitment:"confirmed"});

    console.log("Mint created:", mint.address);

//mintaddress:8tHrfThcwQxwYCgMWR1ubQXHdz4TK8D4yhQLJJF8KWfK
    

}
catch(error){
    console.log(error);
}
})();
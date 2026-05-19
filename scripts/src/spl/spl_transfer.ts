import { address, appendTransactionMessageInstructions, assertIsTransactionWithBlockhashLifetime, createKeyPairSignerFromBytes, createSolanaRpc, createSolanaRpcSubscriptions, createTransactionMessage, getSignatureFromTransaction, sendAndConfirmTransactionFactory, setTransactionMessageFeePayerSigner, setTransactionMessageLifetimeUsingBlockhash, signTransactionMessageWithSigners } from "@solana/kit";
import wallet from "../../devnet-wallet.json" with { type: "json" };
import { findAssociatedTokenPda, 
    getCreateAssociatedTokenInstructionAsync, 
    getMintToInstruction, 
    getTransferCheckedInstruction, TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";

const rpc =createSolanaRpc("https://api.devnet.solana.com");

const rpcSubscription =createSolanaRpcSubscriptions("wss://api.devnet.solana.com");

const mint=address("8tHrfThcwQxwYCgMWR1ubQXHdz4TK8D4yhQLJJF8KWfK");

const to = address("TYdQnGRPKmPgJXeqeQBYFUPpKT3QqzesmdebRaNXXyT");


(async()=> {
    try{

        const signer = await createKeyPairSignerFromBytes(
            new Uint8Array(wallet)
        );


        const sendAndConfirm=sendAndConfirmTransactionFactory({
            rpc,rpcSubscriptions: rpcSubscription,
        });

        const [fromAta] = await findAssociatedTokenPda({
            mint,
            owner:signer.address,
            tokenProgram:TOKEN_PROGRAM_ADDRESS,
        })

        const [toAta] = await findAssociatedTokenPda({
            mint,
            owner:to,
            tokenProgram:TOKEN_PROGRAM_ADDRESS,
        })


        const createAtaTx= await getCreateAssociatedTokenInstructionAsync  ({
            payer:signer,
            mint,
            owner:to
        });

        const transferTx= getTransferCheckedInstruction({
            source:fromAta,
            mint,
            destination:toAta,
            authority:signer,
            amount:1_000_000,
            decimals:6,
        })

       
        
            const {value: latestBlockhash}= await rpc.getLatestBlockhash().send();
        

         const msg=createTransactionMessage({version:0});
            
                const msgWithPayer=setTransactionMessageFeePayerSigner(signer,msg);
            
                const msgWithLifetime= setTransactionMessageLifetimeUsingBlockhash(
                    latestBlockhash,
                    msgWithPayer
                )
        
            const txMessage= appendTransactionMessageInstructions(
                [
                    createAtaTx,
                    transferTx,
                ],
                msgWithLifetime
            );
        
            const signedTx = await signTransactionMessageWithSigners(txMessage);
        
            assertIsTransactionWithBlockhashLifetime(signedTx);
        
            const signature= getSignatureFromTransaction(signedTx);
        
            
        
                await sendAndConfirm(signedTx,{commitment:"confirmed"});
        
                console.log("Minted tokens to ATA. Signature:", signature);

    }
    catch(error){
        console.error("Error:", error);
    }
})()

// signature: 5Zm34TDi8YFyVtbZzRVmp4EBBdVwRmQDrcVgh9fzJa5u2ys69SKYBYYxTgvNm6NKAT7BjKXduDgtBMyGScRS4VNH

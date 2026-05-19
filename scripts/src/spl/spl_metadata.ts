import { createSignerFromKeypair, publicKey, signerIdentity } from "@metaplex-foundation/umi";
import wallet from "../../devnet-wallet.json" with { type: "json" };
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { createMetadataAccountV3, type CreateMetadataAccountV3InstructionAccounts, type CreateMetadataAccountV3InstructionArgs, type DataV2Args } from "@metaplex-foundation/mpl-token-metadata";
import bs58 from "bs58";
const mint =publicKey("8tHrfThcwQxwYCgMWR1ubQXHdz4TK8D4yhQLJJF8KWfK");

const umi = createUmi("https://api.devnet.solana.com");

const keypair=umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));

const signer = createSignerFromKeypair(
    umi, keypair
);

umi.use(signerIdentity(signer));


(async()=>{
    try{

        console.log("Creating metadata account for mint:");

        const accounts:CreateMetadataAccountV3InstructionAccounts={
            mint,
            mintAuthority:signer
        }

        const data: DataV2Args={
            name:"DAIWIK",
            symbol:"DAI",
            uri:"https://arweave.net/1234",
            sellerFeeBasisPoints:1,
            creators:null,
             collection:null,
             uses:null, 
        }
        const args: CreateMetadataAccountV3InstructionArgs={
            data,
            isMutable:true,
            collectionDetails:null,
        }

        const tx= createMetadataAccountV3(umi,{
            ...accounts,
            ...args
        })


        const result=await tx.sendAndConfirm(umi);

        console.log("Metadata account created for mint:", result.signature);

        console.log(bs58.encode(Buffer.from(result.signature)));


    } 
    catch(error){
        console.error("Error:", error);
    }
})();

// signature for metadata
// AU8q8JxFEvPzxUbgg4AgW7NPV2zrsA3SV9MUeDjGfRtrkWHnB2EynJitT2HEfyJH9MnBJGe2zp3cKHvMh6iMN6Y
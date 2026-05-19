import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import wallet from "../../devnet-wallet.json" with { type: "json" };
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys";
import { createSignerFromKeypair, generateSigner, signerIdentity } from "@metaplex-foundation/umi";
import { create, mplCore } from "@metaplex-foundation/mpl-core";
import { base58 } from "@metaplex-foundation/umi/serializers";




const url = "https://api.devnet.solana.com";
const irysurl = "https://devnet.irys.xyz";

const umi = createUmi(url);

const keypair=umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));

const signer=createSignerFromKeypair(umi,keypair);

umi.use(signerIdentity(signer));

umi.use(mplCore());

(async()=>{
    try{

        const metadataUri="https://gateway.irys.xyz/GShV9DBpZ5oncAhXFcMStPjznuAjWK2XVm6fZUWVWLoc";

        const asset= generateSigner(umi);

        const tx = await create(umi,{
            asset,
            name:"Dope",
            uri:metadataUri,
        }).sendAndConfirm(umi);

        const signature = base58.deserialize(tx.signature)[0];

        console.log("minted at", signature);

    }
    catch(e){
        console.error(e);
    }
})()


// minyed at : 2Hd8Co3jfofCarZfP5QqZoFEEwLuPxpJapyYrvw1cyjg14eJDJFzZW5JSsaYBGYkgReeeWrKva18yFNwDRgiJERf
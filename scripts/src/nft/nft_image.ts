import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import wallet from "../../devnet-wallet.json" with { type: "json" };
import { createGenericFile, createSignerFromKeypair, signerIdentity } from "@metaplex-foundation/umi";
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys";
import { readFile } from "fs/promises";


const url = "https://api.devnet.solana.com";
const irysurl = "https://devnet.irys.xyz";


const umi = createUmi(url);

const keypair=umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));

const signer=createSignerFromKeypair(umi,keypair);

umi.use(
    irysUploader({
        address:irysurl,
    })

)

umi.use(signerIdentity(signer));

(async()=>{
try {
    const image = await readFile("./dopehero.png");

    const file = createGenericFile(image, "dopehero.png",{
        contentType:"image/png",
    });

    const [myUri] = await umi.uploader.upload([file]);

    console.log("URI:", myUri);
}

catch(e){
    console.error(e);
}

})()

// imageuro:https://gateway.irys.xyz/3KPmPw6Yp7WUwKL5AEVEtEHfXgoqGidPaWw5mgqbbsP5
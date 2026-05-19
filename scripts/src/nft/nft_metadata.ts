import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import wallet from "../../devnet-wallet.json" with { type: "json" };
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys";
import { createSignerFromKeypair, signerIdentity } from "@metaplex-foundation/umi";




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


(async () => {
    try {
        const image="https://gateway.irys.xyz/3KPmPw6Yp7WUwKL5AEVEtEHfXgoqGidPaWw5mgqbbsP5";

        const metadata = {
            name: "Dope",
            uri: image,
            attributes: [{ trait_type: "Rarity", value: "Legendary" }],


            properties: {
                files: [{
                    type: "image/png",
                    uri: image,
                },

                ],
                category: "image",

            },
        }

        const myUri= await umi.uploader.uploadJson(metadata);
        console.log("Metadata URI:", myUri);






    }
    catch (e) {
        console.log(e)
    }
})()
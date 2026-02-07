import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { WhitelistTransferHook } from "../target/types/whitelist_transfer_hook";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";

(async() => {
 const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

    const program = anchor.workspace.whitelistTransferHook as Program<WhitelistTransferHook>;
  
          const whitelist = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("whitelist"),
      Buffer.from("HynvKzh1nm6ZuWRC1kkKDdSD15Cv2PZJyjamRRQnx9Bm")
      
    ],
    program.programId
  )[0];

  console.log(whitelist)
//HvNi73ZAsXHAA5x6FdZn7r4aTynY6MYuqUsgLmPNXx8V
    
    
})();
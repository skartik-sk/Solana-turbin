use std::io::Read;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList
};
use spl_transfer_hook_interface::solana_msg::msg;

use crate::ID;

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
   pub payer: Signer<'info>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        init,
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump,
        space = ExtraAccountMetaList::size_of(
            InitializeExtraAccountMetaList::extra_account_metas()?.len()
        ).unwrap(),
        payer = payer
    )]
    pub extra_account_meta_list: AccountInfo<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeExtraAccountMetaList<'info> {
    pub fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
        // Derive the whitelist PDA using our program ID
        
         let seed1 = Seed::Literal { bytes: b"whitelist".to_vec() };

         let seed2 = Seed::AccountKey { index: 3 };


            let seed3 = Seed::Literal { bytes: b"whitelist".to_vec() };

         let seed4 = Seed::AccountKey { index: 2 };
        
        //  let (whitelist_pda, _bump) = Pubkey::find_program_address(
        //      &[b"whitelist",b"ALH8UD28X24qwGvG2kpTcogg3Wpvu31FrErpLU8vw6oT"],
             
        //      //todo
        //      &ID
        //  );
        // msg!("whitelisted pda {:?} ", whitelist_pda.as_array());

         msg!("reached. here ");
       
         let datas= ExtraAccountMeta::new_with_seeds(&[seed1,seed2], false, false).unwrap();

         let datas2= ExtraAccountMeta::new_with_seeds(&[seed3,seed4], false, false).unwrap();
         //refer trasfer_hook.rs
        Ok(
            vec![
                datas,
                datas2
                // or
                // ExtraAccountMeta::new_with_pubkey(&whitelist_pda.to_bytes().into(), false, false).unwrap()
            ]
        )
    }
}
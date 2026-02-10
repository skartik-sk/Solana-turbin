use std::io::Read;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::solana_msg::msg;

use crate::constatnt::{EXTRA_META, USER};

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        init,
        seeds = [EXTRA_META.as_bytes(), mint.key().as_ref()],
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

        let seed1 = Seed::Literal {
            bytes: USER.as_bytes().to_vec(),
        };

        let seed2 = Seed::AccountKey { index: 3 };
        msg!("reached. here ");

        let datas = ExtraAccountMeta::new_with_seeds(&[seed1, seed2], false, false).unwrap();

        //refer trasfer_hook.rs
        Ok(vec![
            datas, // or
                  // ExtraAccountMeta::new_with_pubkey(&whitelist_pda.to_bytes().into(), false, false).unwrap()
        ])
    }
}

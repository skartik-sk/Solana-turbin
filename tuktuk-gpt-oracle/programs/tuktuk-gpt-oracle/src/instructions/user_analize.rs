use crate::{instruction, ANCHOR_DISCRIMINATOR};
use crate::{state::Agent, state::AnalysisResult};
use anchor_lang::prelude::*;
use solana_gpt_oracle::AccountMeta as OracleAccountMeta;
use solana_gpt_oracle::{
    cpi::{accounts::InteractWithLlm, interact_with_llm},
    program::SolanaGptOracle,
    ContextAccount, ID,
};

#[derive(Accounts)]
#[instruction(user_pubkey: Pubkey, user_data: String)]
pub struct AnalyzeUser<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Correct interaction account
    #[account(
        mut,
        seeds = [solana_gpt_oracle::Interaction::seed(), payer.key().as_ref(), context_account.key().as_ref()],
        bump,
        seeds::program = oracle_program
    )]
    pub interaction: AccountInfo<'info>,

    #[account(
        seeds = [b"agent"],
        bump = agent.bump
    )]
    pub agent: Account<'info, Agent>,

    /// CHECK: Accept any context
    pub context_account: Account<'info, ContextAccount>,

    /// CHECK: Verified oracle id
    #[account(
        address = ID
    )]
    pub oracle_program: Program<'info, SolanaGptOracle>,

    /// Analysis result account - initialized here so callback can update it
    #[account(
        init_if_needed,
        payer = payer,
        space = ANCHOR_DISCRIMINATOR + AnalysisResult::INIT_SPACE,
        seeds = [b"analysis", user_pubkey.as_ref()],
        bump
    )]
    pub analysis_result: Account<'info, AnalysisResult>,

    pub system_program: Program<'info, System>,
}

impl<'info> AnalyzeUser<'info> {
    pub fn analyse_user(&mut self, user_pubkey: Pubkey, user_data: String) -> Result<()> {
        let cpi_program = self.oracle_program.to_account_info();

        let cpi_accounts = InteractWithLlm {
            payer: self.payer.to_account_info(),
            interaction: self.interaction.to_account_info(),
            context_account: self.context_account.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // passing the user pubkey as string to the llm

        // let callback_discriminator =
        //     anchor_lang::solana_program::hash::hash("global:callback_from_agent".as_bytes())
        //         .to_bytes()[..8]
        //         .try_into()
        //         .unwrap();
        let callback_discriminator = instruction::CallbackFromAgent::DISCRIMINATOR
            .try_into()
            .expect("Incorrect discriminator, it should be of 8 bytes");

        // let (analysis_pda, _bump) =
        //     Pubkey::find_program_address(&[b"analysis", user_pubkey.as_ref()], &crate::ID);

        let metas = vec![
            OracleAccountMeta {
                pubkey: self.analysis_result.key(),
                is_signer: false,
                is_writable: true,
            },
            OracleAccountMeta {
                pubkey: self.payer.key(),
                is_signer: true,
                is_writable: true,
            },
            OracleAccountMeta {
                pubkey: anchor_lang::solana_program::system_program::ID,
                is_signer: false,
                is_writable: false,
            },
        ];

        interact_with_llm(
            cpi_ctx,
            user_data,
            crate::ID,
            callback_discriminator,
            Some(metas),
        )?;

        Ok(())
    }
}
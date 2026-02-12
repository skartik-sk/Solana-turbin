#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;

mod instructions;
mod state;
mod tests;

use instructions::*;

declare_id!("FircrADQ2wgGuvpm8qneNCfKM7o5zoHTWnDQxngpTQ3J");

#[program]
pub mod anchor_escrow {

    use solana_instruction::Instruction;
    // use solana_instruction::Instruction;
    use tuktuk_program::{TransactionSourceV0, TriggerV0, compile_transaction, tuktuk::cpi::{accounts::QueueTaskV0, queue_task_v0}, types::QueueTaskArgsV0};

    use crate::program::AnchorEscrow;

    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.init_escrow(seed, receive, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }

    pub fn take(ctx: Context<Take>,task_id:u16) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        if !((current_time - ctx.accounts.escrow.created_at) >= (5 * 24 * 60 * 60)) {
            let (compiled_tx, _) = compile_transaction(
                vec![Instruction {
                    program_id: crate::ID,
                    accounts: crate::__cpi_client_accounts_take::Take {
                        taker:ctx.accounts.taker.to_account_info(),
                        maker:ctx.accounts.maker.to_account_info(),
                        mint_a:ctx.accounts.mint_a.to_account_info(),
                        mint_b:ctx.accounts.mint_b.to_account_info(),
                        taker_ata_a:ctx.accounts.taker_ata_a.to_account_info(),
                        taker_ata_b:ctx.accounts.taker_ata_b.to_account_info(),
                        maker_ata_b:ctx.accounts.maker_ata_b.to_account_info(),
                        escrow:ctx.accounts.escrow.to_account_info(),
                        vault:ctx.accounts.vault.to_account_info(),
                        associated_token_program:ctx.accounts.associated_token_program.to_account_info(),
                        token_program:ctx.accounts.token_program.to_account_info(),
                        system_program:ctx.accounts.system_program.to_account_info(),
                        task:ctx.accounts.task.to_account_info(),
                        task_queue:ctx.accounts.task_queue.to_account_info(),
                        task_queue_authority:ctx.accounts.task_queue_authority.to_account_info(),
                        queue_authority:ctx.accounts.queue_authority.to_account_info(),
                        tuktuk_program:ctx.accounts.tuktuk_program.to_account_info(),
                        
                    }
                    .to_account_metas(None)
                    .to_vec(),
                    data: crate::instruction::Take{task_id:task_id}.try_to_vec()?,
                }],
                vec![],
            )
            .unwrap();

            queue_task_v0(
                CpiContext::new_with_signer(
                    ctx.accounts.tuktuk_program.to_account_info(),
                    QueueTaskV0 {
                        payer: ctx.accounts.queue_authority.to_account_info(),
                        queue_authority: ctx.accounts.queue_authority.to_account_info(),
                        task_queue: ctx.accounts.task_queue.to_account_info(),
                        task_queue_authority: ctx.accounts.task_queue_authority.to_account_info(),
                        task: ctx.accounts.task.to_account_info(),
                        system_program: ctx.accounts.system_program.to_account_info(),
                    },
                    &[&["queue_authority".as_bytes(), &[ctx.bumps.queue_authority]]],
                ),
                QueueTaskArgsV0 {
                    trigger: TriggerV0::Now,
                    transaction: TransactionSourceV0::CompiledV0(compiled_tx),
                    crank_reward: None,
                    free_tasks: 1,
                    id: task_id,
                    description: "test".to_string(),
                },
            )?;

           
        }
        require!(
            current_time - ctx.accounts.escrow.created_at >= (5 * 24 * 60 * 60),
            CustomError::EscrowLocked
        );
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close_vault()?;

        Ok(())
    }
}

#[error_code]
pub enum CustomError {
    #[msg("You can take escrow only after 5 days")]
    EscrowLocked,
}

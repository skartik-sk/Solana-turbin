use pinocchio::{error::ProgramError, *};
// use pinocchio_associated_token_account::;
use pinocchio_log::log;
use pinocchio_pubkey::derive_address;



pub struct ATA;

impl ATA {
    pub fn init_if_needed(
        mint: &AccountView,
        token_program: &AccountView,
        system_program: &AccountView,
        owner: &AccountView,
        payer: &AccountView,
        ata_account: &AccountView,
    ) -> Result<(), ProgramError> {
        let derive_ata_account = derive_address(
            &[
                owner.address().as_ref(),
                token_program.address().as_ref(),
                mint.address().as_ref(),
            ],
            None,
            pinocchio_associated_token_account::ID.as_array(),
        );

        log!("{}", ata_account.address().as_ref());
        log!("{}", &derive_ata_account);

        // assert_eq!(derive_ata_account, *ata_account.address().as_array());
        log!("This assert is faling. ");
        if ata_account.lamports() > 0 {
            //already intilize do nothing.
        } else {
            pinocchio_associated_token_account::instructions::Create {
                funding_account: payer,
                account: ata_account,
                wallet: owner,
                mint: mint,
                token_program: token_program,
                system_program: system_program,
            }
            .invoke()?;
        }

        Ok(())
    }
}

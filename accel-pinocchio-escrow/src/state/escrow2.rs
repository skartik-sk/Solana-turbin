use bytemuck::{Pod, Zeroable};
use pinocchio::{AccountView, Address, error::ProgramError};
use wincode::{SchemaRead, SchemaWrite};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq,Zeroable,Pod)]
pub struct Escrow2 {
    maker: [u8; 32],
    mint_a: [u8; 32],
    mint_b: [u8; 32],
    amount_to_receive: [u8; 8],
    amount_to_give: [u8; 8],
    pub bump: u8,
}

impl Escrow2 {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 8+1;

    pub fn from_account_info(account_info: &AccountView) -> Result<&mut Self, ProgramError> {
        let mut data = account_info.try_borrow_mut()?;
        if data.len() != Escrow2::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    pub fn _maker(&self) -> pinocchio::Address {
        pinocchio::Address::from(self.maker)
    }

    pub fn set_maker(&mut self, maker: &pinocchio::Address) {
        self.maker.copy_from_slice(maker.as_ref());
    }

    pub fn _mint_a(&self) -> pinocchio::Address {
        pinocchio::Address::from(self.mint_a)
    }

    pub fn set_mint_a(&mut self, mint_a: &pinocchio::Address) {
        self.mint_a.copy_from_slice(mint_a.as_ref());
    }

    pub fn _mint_b(&self) -> pinocchio::Address {
        pinocchio::Address::from(self.mint_b)
    }

    pub fn set_mint_b(&mut self, mint_b: &pinocchio::Address) {
        self.mint_b.copy_from_slice(mint_b.as_ref());
    }

    pub fn _amount_to_receive(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_receive)
    }

    pub fn set_amount_to_receive(&mut self, amount: u64) {
        self.amount_to_receive = amount.to_le_bytes();
    }

    pub fn _amount_to_give(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_give)
    }

    pub fn set_amount_to_give(&mut self, amount: u64) {
        self.amount_to_give = amount.to_le_bytes();
    }

    pub fn set_inner (&mut self ,escrow_acc:&AccountView,maker:&Address,mint_a: &pinocchio::Address,mint_b: &pinocchio::Address,amount_to_receive:[u8;8],amount_to_give:[u8;8],bump:u8 )->Result<(), ProgramError>{
        let offer = Escrow2{
            maker:*maker.as_array(),
            mint_a:*mint_a.as_array(),
            mint_b:*mint_b.as_array(),
            amount_to_receive:amount_to_receive,
            amount_to_give:amount_to_give,
            bump:bump
            
        };
        let offer_bytes = bytemuck::bytes_of(&offer);
                 
              // ? propagates the borrow error → ProgramError
              let mut offer_data = escrow_acc.try_borrow_mut()?;
      
              if offer_data.len() < offer_bytes.len() {
                  return Err(ProgramError::InvalidAccountData);
              }
      
              offer_data[..offer_bytes.len()].copy_from_slice(&offer_bytes);
           
           Ok(())
    }
}
use anchor_lang::prelude::*;
use mpl_core::{
    ID as MPL_CORE_ID, instructions::{
        CreateCollectionV2CpiBuilder,
    }, types::{
        Attribute, Attributes, ExternalCheckResult, ExternalPluginAdapterInitInfo, HookableLifecycleEvent, OracleInitInfo, Plugin, PluginAuthority, PluginAuthorityPair, ValidationResultsOffset
    }
};

#[derive(Accounts)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub collection: Signer<'info>,
    /// CHECK: PDA Update authority
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is the ID of the Metaplex Core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}
impl<'info> CreateCollection<'info> {
    pub fn create_collection(
        &mut self,
        name: String,
        uri: String,
        bumps: &CreateCollectionBumps,
    ) -> Result<()> {
        // Signer seeds for the update authority
        let collection_key = self.collection.key();
        let signer_seeds = &[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ];

        // Derive the oracle PDA address (seeds: [b"oracle"])
        let oracle_pda = Pubkey::find_program_address(&[b"oracle"], &crate::ID).0;

        // Create the collection with CPI builder
        CreateCollectionV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .payer(&self.payer.to_account_info())
            .update_authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugins(vec![PluginAuthorityPair {
                plugin: Plugin::Attributes(Attributes {
                    attribute_list: vec![Attribute {
                        key: "total_staked".to_string(),
                        value: 0.to_string(),
                    }],
                }),
                authority: Some(PluginAuthority::UpdateAuthority),
            }]).external_plugin_adapters(vec![ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
                    base_address: oracle_pda,
                    init_plugin_authority: None,
                    lifecycle_checks: vec![
                        (
                            HookableLifecycleEvent::Transfer,
                            ExternalCheckResult { flags: 4 },
                        ),
                    ],
                    base_address_config: None,
                    results_offset: Some(ValidationResultsOffset::Anchor),
                })])
            .name(name)
            .uri(uri)
            .invoke_signed(&[signer_seeds])?;


        Ok(())
    }
}

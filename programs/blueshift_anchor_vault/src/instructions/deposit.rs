use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{error::VaultError, VAULT_SEED};

#[derive(Accounts)]
pub struct VaultAction<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [VAULT_SEED, signer.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle_deposit(ctx: Context<VaultAction>, amount: u64) -> Result<()> {
    require_eq!(
        ctx.accounts.vault.lamports(),
        0,
        VaultError::VaultAlreadyExists
    );

    require_gt!(
        amount,
        Rent::get()?.minimum_balance(0),
        VaultError::InvalidAmount
    );

    transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            Transfer {
                from: ctx.accounts.signer.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        amount,
    )?;

    Ok(())
}

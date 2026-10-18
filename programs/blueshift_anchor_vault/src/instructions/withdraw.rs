use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{error::VaultError, VaultAction, VAULT_SEED};

pub fn handle_withdraw(ctx: Context<VaultAction>) -> Result<()> {
    require_neq!(ctx.accounts.vault.lamports(), 0, VaultError::InvalidAmount);

    let signer_key = ctx.accounts.signer.key();
    let signer_seeds = &[VAULT_SEED, signer_key.as_ref(), &[ctx.bumps.vault]];

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.key(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.signer.to_account_info(),
            },
            &[&signer_seeds[..]],
        ),
        ctx.accounts.vault.lamports(),
    )?;

    Ok(())
}

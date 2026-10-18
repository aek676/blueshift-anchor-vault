pub mod constants;
pub mod error;
pub mod instructions;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;

declare_id!("3iYuuT56p4FGKEEgvwHj8MbWdPyjGNWiFVXZtQbyyksy");

#[program]
pub mod blueshift_anchor_vault {
    use super::*;

    pub fn deposit(ctx: Context<VaultAction>, amount: u64) -> Result<()> {
        crate::instructions::deposit::handle_deposit(ctx, amount)
    }

    pub fn withdraw(ctx: Context<VaultAction>) -> Result<()> {
        crate::instructions::withdraw::handle_withdraw(ctx)
    }
}

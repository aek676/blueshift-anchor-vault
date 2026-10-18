// Each test file is its own crate and uses only some of these helpers.
#![allow(dead_code)]

use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_native_token::LAMPORTS_PER_SOL,
    solana_signer::Signer,
    solana_transaction::{versioned::VersionedTransaction, InstructionError, TransactionError},
};

pub const INITIAL_BALANCE: u64 = 10 * LAMPORTS_PER_SOL;
pub const DEPOSIT_AMOUNT: u64 = LAMPORTS_PER_SOL;

pub fn setup() -> (LiteSVM, Keypair) {
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/blueshift_anchor_vault.so"
    ));

    let mut svm = LiteSVM::new();
    svm.add_program(blueshift_anchor_vault::id(), bytes)
        .unwrap();

    let signer = Keypair::new();
    svm.airdrop(&signer.pubkey(), INITIAL_BALANCE).unwrap();

    (svm, signer)
}

pub fn vault_pda(owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[blueshift_anchor_vault::VAULT_SEED, owner.as_ref()],
        &blueshift_anchor_vault::id(),
    )
    .0
}

fn vault_action_metas(signer: &Pubkey, vault: Pubkey) -> Vec<anchor_lang::prelude::AccountMeta> {
    blueshift_anchor_vault::accounts::VaultAction {
        signer: *signer,
        vault,
        system_program: system_program::ID,
    }
    .to_account_metas(None)
}

pub fn deposit_ix(signer: &Pubkey, vault: Pubkey, amount: u64) -> Instruction {
    Instruction::new_with_bytes(
        blueshift_anchor_vault::id(),
        &blueshift_anchor_vault::instruction::Deposit { amount }.data(),
        vault_action_metas(signer, vault),
    )
}

pub fn withdraw_ix(signer: &Pubkey, vault: Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        blueshift_anchor_vault::id(),
        &blueshift_anchor_vault::instruction::Withdraw {}.data(),
        vault_action_metas(signer, vault),
    )
}

pub fn send(
    svm: &mut LiteSVM,
    signer: &Keypair,
    instruction: Instruction,
) -> Result<u64, TransactionError> {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&signer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[signer]).unwrap();

    match svm.send_transaction(tx) {
        Ok(meta) => {
            println!("{}", meta.pretty_logs());
            Ok(meta.fee)
        }
        Err(e) => {
            println!("{}", e.meta.pretty_logs());
            Err(e.err)
        }
    }
}

pub fn deposit(
    svm: &mut LiteSVM,
    signer: &Keypair,
    amount: u64,
) -> Result<u64, TransactionError> {
    let ix = deposit_ix(&signer.pubkey(), vault_pda(&signer.pubkey()), amount);
    send(svm, signer, ix)
}

pub fn withdraw(svm: &mut LiteSVM, signer: &Keypair) -> Result<u64, TransactionError> {
    let ix = withdraw_ix(&signer.pubkey(), vault_pda(&signer.pubkey()));
    send(svm, signer, ix)
}

pub fn assert_custom_error(res: Result<u64, TransactionError>, expected: u32) {
    let err = res.expect_err("transaction should have failed");
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::Custom(expected))
    );
}
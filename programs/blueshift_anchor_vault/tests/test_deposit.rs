use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        InstructionData, ToAccountMetas,
    },
    blueshift_anchor_vault::error::VaultError,
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_native_token::LAMPORTS_PER_SOL,
    solana_signer::Signer,
    solana_transaction::{versioned::VersionedTransaction, InstructionError, TransactionError},
};

const INITIAL_BALANCE: u64 = 10 * LAMPORTS_PER_SOL;
const DEPOSIT_AMOUNT: u64 = LAMPORTS_PER_SOL;
// System program error when the source account doesn't have enough lamports.
const SYSTEM_ERROR_INSUFFICIENT_FUNDS: u32 = 1;

fn setup() -> (LiteSVM, Keypair) {
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

fn vault_pda(owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[blueshift_anchor_vault::VAULT_SEED, owner.as_ref()],
        &blueshift_anchor_vault::id(),
    )
    .0
}

fn deposit_ix(signer: &Pubkey, vault: Pubkey, amount: u64) -> Instruction {
    Instruction::new_with_bytes(
        blueshift_anchor_vault::id(),
        &blueshift_anchor_vault::instruction::Deposit { amount }.data(),
        blueshift_anchor_vault::accounts::VaultAction {
            signer: *signer,
            vault,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn send(
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

fn deposit(
    svm: &mut LiteSVM,
    signer: &Keypair,
    amount: u64,
) -> Result<u64, TransactionError> {
    let ix = deposit_ix(&signer.pubkey(), vault_pda(&signer.pubkey()), amount);
    send(svm, signer, ix)
}

fn assert_custom_error(res: Result<u64, TransactionError>, expected: u32) {
    let err = res.expect_err("transaction should have failed");
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::Custom(expected))
    );
}

#[test]
fn test_deposit_transfers_lamports_to_vault() {
    let (mut svm, signer) = setup();
    let vault = vault_pda(&signer.pubkey());

    assert_eq!(svm.get_balance(&vault).unwrap_or(0), 0);

    let fee = deposit(&mut svm, &signer, DEPOSIT_AMOUNT).unwrap();

    assert_eq!(svm.get_balance(&vault).unwrap(), DEPOSIT_AMOUNT);

    // The signer pays the deposit plus the transaction fee.
    let signer_balance = svm.get_balance(&signer.pubkey()).unwrap();
    assert_eq!(signer_balance, INITIAL_BALANCE - DEPOSIT_AMOUNT - fee);

    let vault_account = svm.get_account(&vault).unwrap();
    assert_eq!(vault_account.owner, system_program::ID);
    assert!(vault_account.data.is_empty());
}

#[test]
fn test_deposit_minimum_amount_above_rent() {
    let (mut svm, signer) = setup();
    let amount = svm.minimum_balance_for_rent_exemption(0) + 1;

    deposit(&mut svm, &signer, amount).unwrap();

    assert_eq!(
        svm.get_balance(&vault_pda(&signer.pubkey())).unwrap(),
        amount
    );
}

#[test]
fn test_deposit_fails_if_amount_equals_rent_minimum() {
    let (mut svm, signer) = setup();
    let amount = svm.minimum_balance_for_rent_exemption(0);

    let res = deposit(&mut svm, &signer, amount);

    assert_custom_error(res, VaultError::InvalidAmount.into());
    assert_eq!(
        svm.get_balance(&vault_pda(&signer.pubkey())).unwrap_or(0),
        0
    );
}

#[test]
fn test_deposit_fails_with_zero_amount() {
    let (mut svm, signer) = setup();

    let res = deposit(&mut svm, &signer, 0);

    assert_custom_error(res, VaultError::InvalidAmount.into());
}

#[test]
fn test_deposit_fails_if_vault_already_exists() {
    let (mut svm, signer) = setup();

    deposit(&mut svm, &signer, DEPOSIT_AMOUNT).unwrap();
    svm.expire_blockhash();
    let res = deposit(&mut svm, &signer, DEPOSIT_AMOUNT);

    assert_custom_error(res, VaultError::VaultAlreadyExists.into());
    assert_eq!(
        svm.get_balance(&vault_pda(&signer.pubkey())).unwrap(),
        DEPOSIT_AMOUNT
    );
}

#[test]
fn test_deposit_fails_with_insufficient_funds() {
    let (mut svm, signer) = setup();

    let res = deposit(&mut svm, &signer, INITIAL_BALANCE);

    assert_custom_error(res, SYSTEM_ERROR_INSUFFICIENT_FUNDS);
    assert_eq!(
        svm.get_balance(&vault_pda(&signer.pubkey())).unwrap_or(0),
        0
    );
}

#[test]
fn test_deposit_fails_with_vault_of_another_signer() {
    let (mut svm, signer) = setup();
    let other = Keypair::new();

    let ix = deposit_ix(&signer.pubkey(), vault_pda(&other.pubkey()), DEPOSIT_AMOUNT);
    let res = send(&mut svm, &signer, ix);

    assert_custom_error(res, anchor_lang::error::ErrorCode::ConstraintSeeds.into());
}

#[test]
fn test_deposit_fails_with_non_pda_vault() {
    let (mut svm, signer) = setup();
    let fake_vault = Keypair::new();

    let ix = deposit_ix(&signer.pubkey(), fake_vault.pubkey(), DEPOSIT_AMOUNT);
    let res = send(&mut svm, &signer, ix);

    assert_custom_error(res, anchor_lang::error::ErrorCode::ConstraintSeeds.into());
}

#[test]
fn test_deposits_from_different_signers_use_separate_vaults() {
    let (mut svm, alice) = setup();
    let bob = Keypair::new();
    svm.airdrop(&bob.pubkey(), INITIAL_BALANCE).unwrap();

    deposit(&mut svm, &alice, DEPOSIT_AMOUNT).unwrap();
    deposit(&mut svm, &bob, 2 * DEPOSIT_AMOUNT).unwrap();

    assert_eq!(
        svm.get_balance(&vault_pda(&alice.pubkey())).unwrap(),
        DEPOSIT_AMOUNT
    );
    assert_eq!(
        svm.get_balance(&vault_pda(&bob.pubkey())).unwrap(),
        2 * DEPOSIT_AMOUNT
    );
}

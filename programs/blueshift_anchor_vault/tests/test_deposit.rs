mod common;

use {
    anchor_lang::solana_program::system_program,
    blueshift_anchor_vault::error::VaultError,
    common::*,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

// System program error when the source account doesn't have enough lamports.
const SYSTEM_ERROR_INSUFFICIENT_FUNDS: u32 = 1;

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

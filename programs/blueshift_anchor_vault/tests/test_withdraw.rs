mod common;

use {
    blueshift_anchor_vault::error::VaultError,
    common::*,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

#[test]
fn test_withdraw_returns_all_lamports_to_signer() {
    let (mut svm, signer) = setup();
    let vault = vault_pda(&signer.pubkey());

    let deposit_fee = deposit(&mut svm, &signer, DEPOSIT_AMOUNT).unwrap();
    let withdraw_fee = withdraw(&mut svm, &signer).unwrap();

    assert_eq!(svm.get_balance(&vault).unwrap_or(0), 0);

    let signer_balance = svm.get_balance(&signer.pubkey()).unwrap();
    assert_eq!(signer_balance, INITIAL_BALANCE - deposit_fee - withdraw_fee);
}

#[test]
fn test_withdraw_fails_if_vault_is_empty() {
    let (mut svm, signer) = setup();

    let res = withdraw(&mut svm, &signer);

    assert_custom_error(res, VaultError::InvalidAmount.into());
}

#[test]
fn test_withdraw_fails_if_already_withdrawn() {
    let (mut svm, signer) = setup();

    deposit(&mut svm, &signer, DEPOSIT_AMOUNT).unwrap();
    withdraw(&mut svm, &signer).unwrap();
    svm.expire_blockhash();
    let res = withdraw(&mut svm, &signer);

    assert_custom_error(res, VaultError::InvalidAmount.into());
}

#[test]
fn test_vault_can_be_reused_after_withdraw() {
    let (mut svm, signer) = setup();

    deposit(&mut svm, &signer, DEPOSIT_AMOUNT).unwrap();
    withdraw(&mut svm, &signer).unwrap();
    deposit(&mut svm, &signer, 2 * DEPOSIT_AMOUNT).unwrap();

    assert_eq!(
        svm.get_balance(&vault_pda(&signer.pubkey())).unwrap(),
        2 * DEPOSIT_AMOUNT
    );
}

#[test]
fn test_withdraw_fails_with_vault_of_another_signer() {
    let (mut svm, victim) = setup();
    let attacker = Keypair::new();
    svm.airdrop(&attacker.pubkey(), INITIAL_BALANCE).unwrap();

    deposit(&mut svm, &victim, DEPOSIT_AMOUNT).unwrap();

    let ix = withdraw_ix(&attacker.pubkey(), vault_pda(&victim.pubkey()));
    let res = send(&mut svm, &attacker, ix);

    assert_custom_error(res, anchor_lang::error::ErrorCode::ConstraintSeeds.into());
    assert_eq!(
        svm.get_balance(&vault_pda(&victim.pubkey())).unwrap(),
        DEPOSIT_AMOUNT
    );
}

#[test]
fn test_withdraw_only_empties_own_vault() {
    let (mut svm, alice) = setup();
    let bob = Keypair::new();
    svm.airdrop(&bob.pubkey(), INITIAL_BALANCE).unwrap();

    deposit(&mut svm, &alice, DEPOSIT_AMOUNT).unwrap();
    deposit(&mut svm, &bob, 2 * DEPOSIT_AMOUNT).unwrap();
    withdraw(&mut svm, &bob).unwrap();

    assert_eq!(
        svm.get_balance(&vault_pda(&alice.pubkey())).unwrap(),
        DEPOSIT_AMOUNT
    );
    assert_eq!(svm.get_balance(&vault_pda(&bob.pubkey())).unwrap_or(0), 0);
}

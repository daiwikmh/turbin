use anchor_lang::{
    solana_program::{instruction::Instruction, msg, system_program::ID as SYSTEM_PROGRAM_ID},
    AccountDeserialize, InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use solana_sdk::{
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

fn setup() -> (LiteSVM, Keypair) {
    let program_id = anchor_vault_q2_26::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/anchor_vault_q2_26.so");
    svm.add_program(program_id, bytes);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    (svm, payer)
}

#[test]
fn test_initialize_deposit_withdraw_close() {
    let (mut svm, payer) = setup();

    let user = payer.pubkey();

    let (vault_state_pda, state_bump) =
        Pubkey::find_program_address(&[b"state", user.as_ref()], &anchor_vault_q2_26::id());

    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&[b"vault", vault_state_pda.as_ref()], &anchor_vault_q2_26::id());

    let init_ix = Instruction {
        program_id: anchor_vault_q2_26::id(),
        accounts: anchor_vault_q2_26::accounts::Initialize {
            user,
            vault: vault_pda,
            vault_state: vault_state_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: anchor_vault_q2_26::instruction::Initialize {}.data(),
    };

    let message = Message::new(&[init_ix.clone()], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    let tx1 = svm.send_transaction(transaction).unwrap();

    msg!("Initialize transaction successfll");
    msg!("Transaction :{}", tx1.signature);

    let vault_state_account = svm.get_account(&vault_state_pda).unwrap();
    let vault_state =
        anchor_vault_q2_26::state::VaultState::try_deserialize(&mut vault_state_account.data.as_ref())
            .unwrap();

    assert_eq!(vault_state.vault_bump, vault_bump);
    assert_eq!(vault_state.state_bump, state_bump);

    msg!("First initialize succeeded");

    let message = Message::new(&[init_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let tx = Transaction::new(&[&payer], message, recent_blockhash);

    let result = svm.send_transaction(tx);

    assert!(result.is_err());

    msg!("Second initialize correctly failed");

    let deposit_amount = 1_000_000_000;

    let deposit_ix = Instruction {
        program_id: anchor_vault_q2_26::id(),
        accounts: anchor_vault_q2_26::accounts::Deposit {
            user,
            vault: vault_pda,
            vault_state: vault_state_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: anchor_vault_q2_26::instruction::Deposit {
            amount: deposit_amount,
        }
        .data(),
    };

    let message = Message::new(&[deposit_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction2 = Transaction::new(&[&payer], message, recent_blockhash);

    let tx2 = svm.send_transaction(transaction2).unwrap();

    msg!("Deposit transaction successfll");
    msg!("Transaction :{}", tx2.signature);

    let vault_balance_after_deposit = svm.get_balance(&vault_pda).unwrap();

    assert_eq!(vault_balance_after_deposit, deposit_amount);
    msg!("Balance after deposit: {}", vault_balance_after_deposit);

    let withdraw_amount = 500_000_000;

    let withdraw_ix = Instruction {
        program_id: anchor_vault_q2_26::id(),
        accounts: anchor_vault_q2_26::accounts::Withdraw {
            user,
            vault: vault_pda,
            vault_state: vault_state_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: anchor_vault_q2_26::instruction::Withdraw {
            amount: withdraw_amount,
        }
        .data(),
    };

    let message = Message::new(&[withdraw_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction3 = Transaction::new(&[&payer], message, recent_blockhash);

    let tx3 = svm.send_transaction(transaction3).unwrap();

    msg!("Withdraw transaction successfll");
    msg!("Transaction :{}", tx3.signature);

    let vault_balance_after_withdraw = svm.get_balance(&vault_pda).unwrap();

    assert_eq!(vault_balance_after_withdraw, withdraw_amount);
    msg!("Balance after withdraw. : {}", vault_balance_after_withdraw);

    let close_amount = svm.get_balance(&vault_pda).unwrap();

    let close_ix = Instruction {
        program_id: anchor_vault_q2_26::id(),
        accounts: anchor_vault_q2_26::accounts::Close {
            user,
            vault: vault_pda,
            vault_state: vault_state_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: anchor_vault_q2_26::instruction::Close {}.data(),
    };

    let message = Message::new(&[close_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction4 = Transaction::new(&[&payer], message, recent_blockhash);

    let tx4 = svm.send_transaction(transaction4).unwrap();

    msg!("Close transaction successfll");
    msg!("Transaction :{}", tx4.signature);

    assert!(svm.get_account(&vault_pda).is_none());
    assert!(svm.get_account(&vault_state_pda).is_none());

    let user_balance_after_close = svm.get_balance(&user).unwrap();

    assert!(user_balance_after_close > close_amount);
    msg!("Balance after close. : {}", user_balance_after_close);
}

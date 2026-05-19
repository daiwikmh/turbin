use anchor_lang::{
    solana_program::instruction::Instruction, InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use solana_sdk::{
    message::Message,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[test]
fn test_initialize() {
    let program_id = escrow::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/escrow.so");
    svm.add_program(program_id, bytes);
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &escrow::instruction::Initialize {}.data(),
        escrow::accounts::Initialize {}.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = Transaction::new(&[&payer], msg, blockhash);

    let res = svm.send_transaction(tx);
    assert!(res.is_ok());
}

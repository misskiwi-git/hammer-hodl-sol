use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signer,
    transaction::Transaction,
    system_instruction,
};
use solana_lock_contract::process_instruction;

#[tokio::test]
async fn test_contract() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "solana_lock_contract",
        program_id,
        processor!(process_instruction),
    );

    // 계정 설정
    let payer = Keypair::new();
    let contract_account = Keypair::new();
    let rent = program_test.get_rent().await.unwrap();
    let space = 40; // ContractState 크기
    let lamports = rent.minimum_balance(space);

    program_test.add_account(
        payer.pubkey(),
        Account {
            lamports: 10_000_000,
            ..Account::default()
        },
    );
    program_test.add_account(
        contract_account.pubkey(),
        Account {
            lamports,
            data: vec![0u8; space],
            owner: program_id,
            ..Account::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // 입금 테스트
    let deposit_amount = 1_000_000;
    let mut instruction_data = vec![0];
    instruction_data.extend_from_slice(&deposit_amount.to_le_bytes());
    let deposit_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(contract_account.pubkey(), false),
            AccountMeta::new(payer.pubkey(), true),
        ],
        data: instruction_data,
    };
    let tx = Transaction::new_signed_with_payer(
        &[deposit_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // 출금 실패 테스트 (시간 제한)
    let mut withdraw_data = vec![1];
    withdraw_data.extend_from_slice(&deposit_amount.to_le_bytes());
    let withdraw_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(contract_account.pubkey(), false),
            AccountMeta::new(payer.pubkey(), true),
        ],
        data: withdraw_data,
    };
    let tx = Transaction::new_signed_with_payer(
        &[withdraw_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    assert!(banks_client.process_transaction(tx).await.is_err());
}
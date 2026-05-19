use anchor_lang::{prelude::Pubkey, AccountDeserialize, InstructionData, ToAccountMetas};
use builders_anchor_vault::{accounts, instruction};
use litesvm::LiteSVM;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;

fn program_id() -> Pubkey {
    builders_anchor_vault::ID
}

fn vault_state_pda(user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"state", user.as_ref()], &program_id())
}

fn vault_pda(vault_state: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", vault_state.as_ref()], &program_id())
}

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(program_id(), "../../target/deploy/builders_anchor_vault.so")
        .unwrap();

    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();

    (svm, user)
}

fn send_tx(svm: &mut LiteSVM, instruction: Instruction, signer: &Keypair) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new(&[instruction], Some(&signer.pubkey()));
    let tx = Transaction::new(&[signer], msg, blockhash);
    svm.send_transaction(tx).unwrap();
}

fn initialize_vault(svm: &mut LiteSVM, user: &Keypair, vault_state: Pubkey, vault: Pubkey) {
    send_tx(
        svm,
        Instruction {
            program_id: program_id(),
            accounts: accounts::Initialize {
                user: user.pubkey(),
                vault_state,
                vault,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
            data: instruction::Initialize {}.data(),
        },
        user,
    );
}

#[test]
fn test_initialize() {
    let (mut svm, user) = setup();
    let user_pubkey = user.pubkey();

    let (vault_state, state_bump) = vault_state_pda(&user_pubkey);
    let (vault, vault_bump) = vault_pda(&vault_state);

    initialize_vault(&mut svm, &user, vault_state, vault);

    let vault_state_account = svm.get_account(&vault_state).unwrap();
    let vault_state_data: builders_anchor_vault::VaultState =
        AccountDeserialize::try_deserialize(&mut &vault_state_account.data[..]).unwrap();

    assert_eq!(vault_state_data.vault_bump, vault_bump);
    assert_eq!(vault_state_data.state_bump, state_bump);

    // Vault is a system account (no init), so it starts with 0 lamports
    assert!(svm.get_account(&vault).is_none());
}

#[test]
fn test_deposit() {
    let (mut svm, user) = setup();
    let user_pubkey = user.pubkey();

    let (vault_state, _) = vault_state_pda(&user_pubkey);
    let (vault, _) = vault_pda(&vault_state);

    initialize_vault(&mut svm, &user, vault_state, vault);

    let deposit_amount: u64 = 1_000_000_000; // 1 SOL

    let initial_vault_balance = svm.get_account(&vault).map(|a| a.lamports).unwrap_or(0);
    let initial_user_balance = svm.get_account(&user_pubkey).unwrap().lamports;

    send_tx(
        &mut svm,
        Instruction {
            program_id: program_id(),
            accounts: accounts::Deposit {
                user: user_pubkey,
                vault,
                vault_state,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
            data: instruction::Deposit {
                amount: deposit_amount,
            }
            .data(),
        },
        &user,
    );

    let final_vault_balance = svm.get_account(&vault).unwrap().lamports;
    let final_user_balance = svm.get_account(&user_pubkey).unwrap().lamports;

    assert_eq!(final_vault_balance, initial_vault_balance + deposit_amount);
    // User pays deposit + tx fee (5000 lamports)
    assert_eq!(
        final_user_balance,
        initial_user_balance - deposit_amount - 5000
    );
}

#[test]
fn test_withdraw() {
    let (mut svm, user) = setup();
    let user_pubkey = user.pubkey();

    let (vault_state, _) = vault_state_pda(&user_pubkey);
    let (vault, _) = vault_pda(&vault_state);

    initialize_vault(&mut svm, &user, vault_state, vault);

    // Deposit 1 SOL
    send_tx(
        &mut svm,
        Instruction {
            program_id: program_id(),
            accounts: accounts::Deposit {
                user: user_pubkey,
                vault,
                vault_state,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
            data: instruction::Deposit {
                amount: 1_000_000_000,
            }
            .data(),
        },
        &user,
    );

    let withdraw_amount: u64 = 500_000_000; // 0.5 SOL

    let initial_vault_balance = svm.get_account(&vault).unwrap().lamports;
    let initial_user_balance = svm.get_account(&user_pubkey).unwrap().lamports;

    send_tx(
        &mut svm,
        Instruction {
            program_id: program_id(),
            accounts: accounts::Withdraw {
                user: user_pubkey,
                vault,
                vault_state,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
            data: instruction::Withdraw {
                amount: withdraw_amount,
            }
            .data(),
        },
        &user,
    );

    let final_vault_balance = svm.get_account(&vault).unwrap().lamports;
    let final_user_balance = svm.get_account(&user_pubkey).unwrap().lamports;

    assert_eq!(final_vault_balance, initial_vault_balance - withdraw_amount);
    // User receives withdrawal minus tx fee
    assert_eq!(
        final_user_balance,
        initial_user_balance + withdraw_amount - 5000
    );
}

#[test]
fn test_close() {
    let (mut svm, user) = setup();
    let user_pubkey = user.pubkey();

    let (vault_state, _) = vault_state_pda(&user_pubkey);
    let (vault, _) = vault_pda(&vault_state);

    initialize_vault(&mut svm, &user, vault_state, vault);

    // Deposit 1 SOL
    send_tx(
        &mut svm,
        Instruction {
            program_id: program_id(),
            accounts: accounts::Deposit {
                user: user_pubkey,
                vault,
                vault_state,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
            data: instruction::Deposit {
                amount: 1_000_000_000,
            }
            .data(),
        },
        &user,
    );

    // Withdraw 0.5 SOL
    send_tx(
        &mut svm,
        Instruction {
            program_id: program_id(),
            accounts: accounts::Withdraw {
                user: user_pubkey,
                vault,
                vault_state,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
            data: instruction::Withdraw {
                amount: 500_000_000,
            }
            .data(),
        },
        &user,
    );

    let initial_vault_balance = svm.get_account(&vault).unwrap().lamports;
    let initial_vault_state_balance = svm.get_account(&vault_state).unwrap().lamports;
    let initial_user_balance = svm.get_account(&user_pubkey).unwrap().lamports;

    send_tx(
        &mut svm,
        Instruction {
            program_id: program_id(),
            accounts: accounts::Close {
                user: user_pubkey,
                vault,
                vault_state,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
            data: instruction::Close {}.data(),
        },
        &user,
    );

    // Vault should be gone
    assert!(svm.get_account(&vault).is_none());

    // VaultState should be closed
    assert!(svm.get_account(&vault_state).is_none());

    // User gets back all remaining lamports minus tx fee
    let final_user_balance = svm.get_account(&user_pubkey).unwrap().lamports;
    assert_eq!(
        final_user_balance,
        initial_user_balance + initial_vault_balance + initial_vault_state_balance - 5000
    );
}

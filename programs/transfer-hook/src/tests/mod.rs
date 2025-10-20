#[cfg(test)]
mod tests {
    use anchor_lang::prelude::{Account, AccountInfo, InterfaceAccount, Program, System};
    use {
        anchor_lang::{
            prelude::msg,
            solana_program::program_pack::Pack,
            AccountDeserialize,
            InstructionData,
            ToAccountMetas
        },
        anchor_spl::{
            token_2022::spl_token_2022::{
                ID as TOKEN_PROGRAM_ID,
                extension::{
                    transfer_hook::TransferHook as TransferHookExt,
                    BaseStateWithExtensions,
                    StateWithExtensions,
                    ExtensionType,
                    interest_bearing_mint::InterestBearingConfig,
                },
                state::Mint as Token2022Mint,
            },
            associated_token::{
                self,
                spl_associated_token_account
            },
            token::spl_token
        },
        litesvm::LiteSVM,
        litesvm_token::{
            // spl_token::ID as TOKEN_PROGRAM_ID,
            CreateAssociatedTokenAccount,
            CreateMint, MintTo
        },
        solana_rpc_client::rpc_client::RpcClient,
       // solana_account::{Account, ReadableAccount},
        solana_instruction::Instruction,
        solana_keypair::Keypair,
        solana_message::Message,
        solana_native_token::LAMPORTS_PER_SOL,
        solana_pubkey::Pubkey,
        solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID,
        solana_signer::Signer,
        solana_transaction::Transaction,
        solana_address::Address,
        std::{
            path::PathBuf,
            str::FromStr
        }
    };
    use anchor_lang::solana_program::sysvar::clock::Clock;
    use anchor_spl::token_interface::Mint;
    use crate::instructions::TransferHook;
    use crate::state::Whitelist;

    static PROGRAM_ID: Pubkey = crate::ID;

    fn setup() -> (LiteSVM, Keypair) {
        // Initialize LiteSVM and payer
        let mut program = LiteSVM::new();
        let payer = Keypair::new();

        // Airdrop some SOL to the payer keypair
        program
            .airdrop(&payer.pubkey(), 50 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");

        // Load program SO file
        let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/deploy/transfer_hook.so");

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        let _ = program.add_program(PROGRAM_ID, &program_data);

        // Return the LiteSVM instance and payer keypair
        (program, payer)
    }

    #[test]
    fn test_add_to_whitelist() {
        let (mut program,payer) = setup();
        let user = Keypair::new();
        let mint = Keypair::new();

        let (whitelist_acc, w_bump ) = Pubkey::find_program_address(
            &["whitelist".as_bytes(), &mint.pubkey().as_ref(), &user.pubkey().as_ref()],
            &crate::ID,
        );

        let accounts = crate::accounts::WhitelistOperations{
            admin:payer.pubkey(),
            address: user.pubkey(),
            mint: mint.pubkey(),
            whitelist_PDA: whitelist_acc,
            system_program : SYSTEM_PROGRAM_ID
        };

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: crate::instruction::AddToWhitelist {}.data(),
        };

        let transaction = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer],
            program.latest_blockhash(),
        );

        let res = program.send_transaction(transaction);

        match &res {
            Ok(tx) => {
                msg!("\n\ntest add to whitelist: transaction successful:\nLogs:\n{:?}", tx.logs);
                msg!("CUs Consumed: {}", tx.compute_units_consumed);
            }
            Err(err) => {
                msg!("\n\ntest_deposit: transaction failed with {:?}", err);
            }
        }

        assert!(res.is_ok(), "Add to whitelist failed");
    }

    #[test]
    fn test_remove_from_whitelist() {
        let (mut program,payer) = setup();
        let user = Keypair::new();
        let mint = Keypair::new();

        let (whitelist_acc, w_bump ) = Pubkey::find_program_address(
            &["whitelist".as_bytes(), &mint.pubkey().as_ref(), &user.pubkey().as_ref()],
            &crate::ID,
        );

        let accounts = crate::accounts::WhitelistOperations{
            admin:payer.pubkey(),
            address: user.pubkey(),
            mint: mint.pubkey(),
            whitelist_PDA: whitelist_acc,
            system_program : SYSTEM_PROGRAM_ID
        };

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: crate::instruction::AddToWhitelist {}.data(),
        };

        let transaction = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer],
            program.latest_blockhash(),
        );

        let res = program.send_transaction(transaction);

        match &res {
            Ok(tx) => {
                msg!("\n\ntest add to whitelist: transaction successful:\nLogs:\n{:?}", tx.logs);
                msg!("CUs Consumed: {}", tx.compute_units_consumed);
            }
            Err(err) => {
                msg!("\n\ntest_deposit: transaction failed with {:?}", err);
            }
        }

        assert!(res.is_ok(), "Add to whitelist failed");

        let accounts = crate::accounts::WhitelistOperations{
            admin:payer.pubkey(),
            address: user.pubkey(),
            mint: mint.pubkey(),
            whitelist_PDA: whitelist_acc,
            system_program : SYSTEM_PROGRAM_ID
        };

        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: crate::instruction::RemoveFromWhitelist {}.data(),
        };

        let transaction = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer],
            program.latest_blockhash(),
        );

        let res = program.send_transaction(transaction);

        match &res {
            Ok(tx) => {
                msg!("\n\ntest remove from whitelist: transaction successful:\nLogs:\n{:?}", tx.logs);
                msg!("CUs Consumed: {}", tx.compute_units_consumed);
            }
            Err(err) => {
                msg!("\n\ntest_remove_from_whitelist: transaction failed with {:?}", err);
            }
        }

        let wl_account = program.get_account(&whitelist_acc);
        assert!(wl_account.is_none(), "Remove from whitelist failed");
    }
}
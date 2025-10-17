use anchor_lang::prelude::Pubkey;

#[cfg(test)]
mod tests {
    use super::*;
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
                ID as ASSOCIATED_TOKEN_PROGRAM_ID,
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
        solana_account::{Account, ReadableAccount},
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
    use crate::instructions::InitializeVault;
    use transfer_hook;

    static PROGRAM_ID: Pubkey = crate::ID;
    
    
    fn setup() -> (LiteSVM, Keypair) {
        // Initialize LiteSVM and payer
        let mut program = LiteSVM::new();
        let payer = Keypair::new();

        let TRANSFER_HOOK_PROGRAM_ID = transfer_hook::ID;
        msg!("TRANSFER_HOOK_PROGRAM_ID : {:?}", TRANSFER_HOOK_PROGRAM_ID);
        msg!("VAULT_PROGRAM_ID : {:?}", PROGRAM_ID);

        // Airdrop some SOL to the payer keypair
        program
            .airdrop(&payer.pubkey(), 50 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");

        // Load program SO file
        let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/deploy/interest_bearing_vault.so");

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        let _ = program.add_program(PROGRAM_ID, &program_data);

        // Load program SO file
        let so_path1 = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/deploy/transfer_hook.so");

        let program_data1 = std::fs::read(so_path1).expect("Failed to read program SO file");

        let _ = program.add_program(TRANSFER_HOOK_PROGRAM_ID, &program_data1);

        // Return the LiteSVM instance and payer keypair
        (program, payer)
    }


    #[test]
    fn test_create_interest_bearing_mint() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let (mut program, payer) = setup();

        let payer_pubkey = payer.pubkey();

        let asspciated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        // Create mint keypair
        let mint = Keypair::new();
        let mint_pubkey = mint.pubkey();

        // Find extra account meta list PDA (not used in this test, but needed for accounts)
        let (extra_account_meta_list, _bump) = Pubkey::find_program_address(
            &[b"extra-account-metas", mint_pubkey.as_ref()],
            &PROGRAM_ID,
        );

        // Interest rate: 500 basis points = 5% APY
        let interest_rate: i16 = 500;

        // Build the instruction
        let accounts = crate::accounts::TokenFactory {
            user: payer_pubkey,
            mint: mint_pubkey,
            extra_account_meta_list,
            system_program: SYSTEM_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
        };

        let instruction_data = crate::instruction::CreateMintWithExtensions {
            interest_rate,
        };

        let init_mint_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: instruction_data.data(),
        };

        // Create and send transaction
        let recent_blockhash = program.latest_blockhash();
        let transaction = Transaction::new_signed_with_payer(
            &[init_mint_ix],
            Some(&payer_pubkey),
            &[&payer, &mint],
            recent_blockhash,
        );

        // let message = Message::new(&[init_mint_ix], Some(&payer.pubkey()));
        // let recent_blockhash = program.latest_blockhash();
        // let transaction = Transaction::new(&[&payer, &mint], message, recent_blockhash);

        let tx_result = program.send_transaction(transaction);
        let mut ok = false;
        match tx_result {
            Ok(tx) => {
                // Log transaction details
                msg!("\n\ntest_create_interest_bearing_mint:  transaction successful");
                msg!("CUs Consumed: {}", tx.compute_units_consumed);
                msg!("Tx Signature: {}", tx.signature);
                msg!("Tx Logs: {:?}", tx.logs);
                ok = true;
            },

            Err(err) => {
                msg!("\n\ntest_take transaction failed with {:?}", err);
            }
        }
        // assert!(ok, "Transaction failed: {:?}", tx_result.err());

        // Verify the mint was created with interest bearing extension
        let mint_account = program
            .get_account(&mint_pubkey)
            .expect("Mint account should exist");

        // Verify account is owned by Token-2022
        assert_eq!(mint_account.owner, TOKEN_PROGRAM_ID);

        // Deserialize and verify the mint state
        let mint_data = mint_account.data.as_slice();
        let mint_state = StateWithExtensions::<Token2022Mint>::unpack(mint_data)
            .expect("Failed to unpack mint state");

        // Verify basic mint properties
        assert_eq!(mint_state.base.decimals, 9, "Decimals should be 9");
        assert_eq!(
            mint_state.base.mint_authority.unwrap(),
            payer_pubkey,
            "Mint authority should be payer"
        );
        assert_eq!(
            mint_state.base.freeze_authority.unwrap(),
            payer_pubkey,
            "Freeze authority not eshould be"
        );

        // Verify interest bearing extension
        let interest_config = mint_state
            .get_extension::<InterestBearingConfig>()
            .expect("Interest bearing extension should exist");

        assert_eq!(
            interest_config.rate_authority.0,
            payer_pubkey,
            "Rate authority should be payer"
        );
        assert_eq!(
            interest_config.current_rate,
            interest_rate.into(),
            "Interest rate should be 500 basis points"
        );

        println!("✅ Mint created successfully with interest bearing extension");
        println!("   Mint: {}", mint_pubkey);
        println!("   Interest Rate: {} basis points ({}%)", interest_rate, interest_rate as f64 / 100.0);
        println!("   Rate Authority: {}", payer_pubkey);

        let transfer_hook_config = mint_state
            .get_extension::<TransferHookExt>()
            .expect("Interest bearing extension should exist");

        println!("✅ Transfer Hook:");
        println!("   Program ID: {:?}", transfer_hook_config.program_id);
        println!("   Authority: {:?}", transfer_hook_config.authority);
    }

    #[test]
    fn test_initialize_vault() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let (mut program, payer) = setup();
        let TRANSFER_HOOK_PROGRAM_ID = transfer_hook::ID;

        let payer_pubkey = payer.pubkey();

        let asspciated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        // Create mint keypair
        let mint = Keypair::new();

        // Find extra account meta list PDA (not used in this test, but needed for accounts)
        let (extra_account_meta_list, _bump) = Pubkey::find_program_address(
            &[b"extra-account-metas", mint.pubkey().as_ref()],
            &PROGRAM_ID,
        );

        // Interest rate: 500 basis points = 5% APY
        let interest_rate: i16 = 500;

        // Build the instruction
        let accounts = crate::accounts::TokenFactory {
            user: payer_pubkey,
            mint: mint.pubkey(),
            extra_account_meta_list,
            system_program: SYSTEM_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
        };

        let instruction_data = crate::instruction::CreateMintWithExtensions {
            interest_rate,
        };

        let init_mint_ix = Instruction {
            program_id: TRANSFER_HOOK_PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: instruction_data.data(),
        };

        // Create and send transaction
        let recent_blockhash = program.latest_blockhash();
        let transaction = Transaction::new_signed_with_payer(
            &[init_mint_ix],
            Some(&payer_pubkey),
            &[&payer, &mint],
            recent_blockhash,
        );

        // let message = Message::new(&[init_mint_ix], Some(&payer.pubkey()));
        // let recent_blockhash = program.latest_blockhash();
        // let transaction = Transaction::new(&[&payer, &mint], message, recent_blockhash);

        let tx_result = program.send_transaction(transaction);
        let mut ok = false;
        match tx_result {
            Ok(tx) => {
                // Log transaction details
                msg!("\n\ntest_init_vault: create_interest_bearing_mint:  transaction successful");
                msg!("CUs Consumed: {}", tx.compute_units_consumed);
                msg!("Tx Signature: {}", tx.signature);
                msg!("Tx Logs: {:?}", tx.logs);
                ok = true;
            },

            Err(err) => {
                msg!("\n\ntest_init_vault: transaction failed with {:?}", err);
            }
        }
        // assert!(ok, "Transaction failed: {:?}", tx_result.err());

        // Verify the mint was created with interest bearing extension
        let mint_account = program
            .get_account(&mint.pubkey())
            .expect("Mint account should exist");

        // Verify account is owned by Token-2022
        assert_eq!(mint_account.owner, TOKEN_PROGRAM_ID);

        // Deserialize and verify the mint state
        let mint_data = mint_account.data.as_slice();
        let mint_state = StateWithExtensions::<Token2022Mint>::unpack(mint_data)
            .expect("Failed to unpack mint state");

        // Verify basic mint properties
        assert_eq!(mint_state.base.decimals, 9, "Decimals should be 9");
        assert_eq!(
            mint_state.base.mint_authority.unwrap(),
            payer_pubkey,
            "Mint authority should be payer"
        );
        assert_eq!(
            mint_state.base.freeze_authority.unwrap(),
            payer_pubkey,
            "Freeze authority not eshould be"
        );

        // Verify interest bearing extension
        let interest_config = mint_state
            .get_extension::<InterestBearingConfig>()
            .expect("Interest bearing extension should exist");

        assert_eq!(
            interest_config.rate_authority.0,
            payer_pubkey,
            "Rate authority should be payer"
        );
        assert_eq!(
            interest_config.current_rate,
            interest_rate.into(),
            "Interest rate should be 500 basis points"
        );

        println!("✅ Mint created successfully with interest bearing extension");
        println!("   Mint: {}", mint.pubkey());
        println!("   Interest Rate: {} basis points ({}%)", interest_rate, interest_rate as f64 / 100.0);
        println!("   Rate Authority: {}", payer_pubkey);

        let transfer_hook_config = mint_state
            .get_extension::<TransferHookExt>()
            .expect("Interest bearing extension should exist");

        println!("✅ Transfer Hook:");
        println!("   Program ID: {:?}", transfer_hook_config.program_id);
        println!("   Authority: {:?}", transfer_hook_config.authority);
        
        // Init vault process start
        
        let vault_seeds = &[b"vault", payer_pubkey.as_ref()];

        let (vault_PDA, v_bump) = Pubkey::find_program_address(
            &[b"vault", payer_pubkey.as_ref()],
            &PROGRAM_ID,
        );
        msg!("test_init_vault: vault PDA: {}\n", vault_PDA);

        // Create the vault's associated token account for Mint 
        let reserve_ata  = associated_token::get_associated_token_address_with_program_id(&vault_PDA, &mint.pubkey(), &TRANSFER_HOOK_PROGRAM_ID);

        msg!("test_init_vault: vault ATA: {}\n", reserve_ata);

        msg!("test_init_vault: payer_pubkey: {}\n", payer_pubkey);

        msg!("test_init_vault: mint_pubkey: {}\n", mint.pubkey());

        // Build the instruction
        let accounts = crate::accounts::InitializeVault {
            vault_authority: payer_pubkey,
            mint: mint.pubkey(),
            hook_program_id: TRANSFER_HOOK_PROGRAM_ID,
            vault:vault_PDA,
            token_reserve:reserve_ata,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        };

        // let instruction_data = crate::instruction::InitializeVault {};

        let init_vault_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: accounts.to_account_metas(None),
            data: crate::instruction::InitializeVault{}.data(),
        };

        // Create and send transaction
        let recent_blockhash = program.latest_blockhash();
        let transaction = Transaction::new_signed_with_payer(
            &[init_vault_ix],
            Some(&payer_pubkey),
            &[&payer],
            recent_blockhash,
        );

        // let message = Message::new(&[init_mint_ix], Some(&payer.pubkey()));
        // let recent_blockhash = program.latest_blockhash();
        // let transaction = Transaction::new(&[&payer, &mint], message, recent_blockhash);

        let tx_result = program.send_transaction(transaction);
        let mut ok = false;
        match tx_result {
            Ok(tx) => {
                // Log transaction details
                msg!("\n\ntest_initialize_vault:  transaction successful");
                msg!("CUs Consumed: {}", tx.compute_units_consumed);
                msg!("Tx Signature: {}", tx.signature);
                msg!("Tx Logs: {:?}", tx.logs);
                ok = true;
            },

            Err(err) => {
                msg!("\n\ntest_initialize_vault: transaction failed with {:?}", err);
            }
        }
        
    }
}
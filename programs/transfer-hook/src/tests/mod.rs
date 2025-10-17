use anchor_lang::prelude::Pubkey;

#[cfg(test)]
mod tests {
    use {
        anchor_lang::{
            prelude::msg,
            solana_program::program_pack::Pack,
            AccountDeserialize,
            InstructionData,
            ToAccountMetas
        }, anchor_spl::{
            token_2022::spl_token_2022::extension::interest_bearing_mint::InterestBearingConfig,
            associated_token::{
                self,
                spl_associated_token_account
            },
            token::spl_token
        },
        litesvm::LiteSVM,
        litesvm_token::{
            spl_token::ID as TOKEN_PROGRAM_ID,
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
            token_program: spl_token_2022::ID,
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
        // let recent_blockhash = program.latest_blockhash();
        // let transaction = Transaction::new_signed_with_payer(
        //     &[init_mint_ix],
        //     Some(&payer_pubkey),
        //     &[&payer, &mint],
        //     recent_blockhash,
        // );

        let message = Message::new(&[init_mint_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();
        let transaction = Transaction::new(&[&payer, &mint], message, recent_blockhash);

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
        assert!(ok, "Transaction failed: {:?}", tx_result.err());

        // Verify the mint was created with interest bearing extension
        let mint_account = program
            .get_account(&mint_pubkey)
            .expect("Mint account should exist");

        // Verify account is owned by Token-2022
        assert_eq!(mint_account.owner, spl_token_2022::ID);

        // Deserialize and verify the mint state
        let mint_data = mint_account.data.as_slice();
        let mint_state = StateWithExtensions::<Mint>::unpack(mint_data)
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

        use spl_token_2022::{
            extension::{BaseStateWithExtensions, StateWithExtensions, ExtensionType},
            state::Mint,
        };
        
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
    }

   // #[test]
    // fn test_interest_rate_calculations() {
    //     // Initialize LiteSVM
    //     let mut svm = LiteSVM::new();
    // 
    //     let program_id = Pubkey::new_unique();
    //     let program_data = std::fs::read("target/deploy/your_program.so")
    //         .expect("Failed to read program file");
    //     svm.add_program(program_id, &program_data);
    // 
    //     let payer = Keypair::new();
    //     svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    // 
    //     // Test different interest rates
    //     let test_rates = vec![
    //         (100, "1%"),    // 1% APY
    //         (500, "5%"),    // 5% APY
    //         (1000, "10%"),  // 10% APY
    //         (5000, "50%"),  // 50% APY
    //         (-100, "-1%"),  // Negative 1% (deflationary)
    //     ];
    // 
    //     for (rate, description) in test_rates {
    //         let mint = Keypair::new();
    //         let (extra_account_meta_list, _) = Pubkey::find_program_address(
    //             &[b"extra-account-metas", mint.pubkey().as_ref()],
    //             &program_id,
    //         );
    // 
    //         let accounts = transfer_hook::accounts::TokenFactory {
    //             user: payer.pubkey(),
    //             mint: mint.pubkey(),
    //             extra_account_meta_list,
    //             system_program: solana_sdk::system_program::ID,
    //             token_program: spl_token_2022::ID,
    //         };
    // 
    //         let instruction_data = transfer_hook::instruction::CreateMintWithExtensions {
    //             interest_rate: rate,
    //         };
    // 
    //         let instruction = Instruction {
    //             program_id,
    //             accounts: accounts.to_account_metas(None),
    //             data: instruction_data.data(),
    //         };
    // 
    //         let recent_blockhash = svm.latest_blockhash();
    //         let transaction = Transaction::new_signed_with_payer(
    //             &[instruction],
    //             Some(&payer.pubkey()),
    //             &[&payer, &mint],
    //             recent_blockhash,
    //         );
    // 
    //         let result = svm.send_transaction(transaction);
    //         assert!(result.is_ok(), "Failed to create mint with rate {}: {:?}", description, result.err());
    // 
    //         // Verify the interest rate was set correctly
    //         let mint_account = svm.get_account(&mint.pubkey()).unwrap();
    //         let mint_state = StateWithExtensions::<Mint>::unpack(&mint_account.data).unwrap();
    //         let interest_config = mint_state.get_extension::<InterestBearingConfig>().unwrap();
    // 
    //         assert_eq!(interest_config.current_rate, rate);
    //         println!("✅ Successfully created mint with {} APY (rate: {})", description, rate);
    //     }
    // }
    // 
    // #[test]
    // fn test_mint_properties_without_transfer_hook() {
    //     // Test creating a mint with ONLY interest bearing extension
    //     // (no transfer hook in this test)
    // 
    //     let mut svm = LiteSVM::new();
    //     let program_id = Pubkey::new_unique();
    //     let program_data = std::fs::read("target/deploy/transfer_hook.so")
    //         .expect("Failed to read program file");
    //     svm.add_program(program_id, &program_data);
    // 
    //     let payer = Keypair::new();
    //     svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    // 
    //     let mint = Keypair::new();
    //     let (extra_account_meta_list, _) = Pubkey::find_program_address(
    //         &[b"extra-account-metas", mint.pubkey().as_ref()],
    //         &program_id,
    //     );
    // 
    //     let interest_rate = 750; // 7.5% APY
    // 
    //     let accounts = transfer_hook::accounts::TokenFactory {
    //         user: payer.pubkey(),
    //         mint: mint.pubkey(),
    //         extra_account_meta_list,
    //         system_program: solana_sdk::system_program::ID,
    //         token_program: spl_token_2022::ID,
    //     };
    // 
    //     let instruction_data = transfer_hook::instruction::CreateMintWithExtensions {
    //         interest_rate,
    //     };
    // 
    //     let instruction = Instruction {
    //         program_id,
    //         accounts: accounts.to_account_metas(None),
    //         data: instruction_data.data(),
    //     };
    // 
    //     let recent_blockhash = svm.latest_blockhash();
    //     let transaction = Transaction::new_signed_with_payer(
    //         &[instruction],
    //         Some(&payer.pubkey()),
    //         &[&payer, &mint],
    //         recent_blockhash,
    //     );
    // 
    //     svm.send_transaction(transaction).unwrap();
    // 
    //     // Detailed verification
    //     let mint_account = svm.get_account(&mint.pubkey()).unwrap();
    //     let mint_state = StateWithExtensions::<Mint>::unpack(&mint_account.data).unwrap();
    // 
    //     // Verify mint supply starts at 0
    //     assert_eq!(mint_state.base.supply, 0, "Initial supply should be 0");
    // 
    //     // Verify interest bearing config
    //     let interest_config = mint_state.get_extension::<InterestBearingConfig>().unwrap();
    //     assert_eq!(interest_config.current_rate, interest_rate);
    // 
    //     // Verify initialization timestamp exists
    //     assert!(
    //         interest_config.initialization_timestamp > 0,
    //         "Initialization timestamp should be set"
    //     );
    // 
    //     println!("✅ All mint properties verified");
    //     println!("   Supply: {}", mint_state.base.supply);
    //     println!("   Decimals: {}", mint_state.base.decimals);
    //     println!("   Interest Rate: {} basis points", interest_config.current_rate);
    //     println!("   Initialized at: {}", interest_config.initialization_timestamp);
    // }
}
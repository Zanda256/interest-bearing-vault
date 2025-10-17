use anchor_lang::error_code;

#[error_code]
pub enum WhitelistError {
    #[msg("Invalid whitelist account")]
    InvalidWhitelistAccount,

    #[msg("Account already exists")]
    AccountAlreadyExists,

    #[msg("Account does not exist")]
    AccountDoesNotExist,

    #[msg("Whitelist PDA does not expected PDA")]
    AccountDoesNotMatch,

    #[msg("Whitelist PDA does not exist")]
    AccountNotWhitelisted,

    #[msg("Failed to deserialize whitelist data")]
    DeserializeWhitelistData,
}

#[error_code]
pub enum ExtraAccountMetaError {
    #[msg("Invalid ExtraAccountMeta provided")]
    InvalidExtraAccountMeta,
}
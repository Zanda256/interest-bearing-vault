use anchor_lang::error_code;

#[error_code]
pub enum ExtraAccountMetaError {
    #[msg("Invalid ExtraAccountMeta provided")]
    InvalidExtraAccountMeta,
}

#[error_code]
pub enum VaultError {
    #[msg("Invalid amount: must be greater than 0")]
    InvalidAmount,
    #[msg("Insufficient funds in vault")]
    InsufficientFunds,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Arithmetic underflow")]
    Underflow,
    #[msg("Unauthorized: only vault authority can perform this action")]
    Unauthorized,
}

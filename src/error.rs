use num_enum::{FromPrimitive, IntoPrimitive};
use solana_program::program_error::ProgramError;
use thiserror::Error;

pub type TradeBotResult<T> = Result<T, TradeBotErrors>;

#[derive( Debug, PartialEq, Eq)]
pub enum TradeBotError {
    Errors(TradeBotErrors),

    ProgramError(ProgramError),

}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TradeBotErrors {
    #[error("Instruction Is not known by the program(a.k.a me)")]
    UnknownInstruction,
    #[error("Instruction data is invalid")]
    InvalidInstruction,
    #[error("The market address is already saved")]
    MarketAlreadyInitialized,
    #[error("The provided market is not initialized")]
    MarketNotKnown,
    #[error("You are not authorized to perform this action")]
    Unauthorized,
    #[error("Trader already exists")]
    TraderExists,
    #[error("Not enough tokens")]
    InsufficientTokens,
    #[error("Program Error")]
    ProgramErr(ProgramError),
    #[error("Unknown error")]
    UnknownError,


}



impl From<TradeBotErrors> for ProgramError {
    fn from(e: TradeBotErrors) -> ProgramError {
        ProgramError::Custom(1)
    }
}

impl From<ProgramError> for TradeBotErrors {
    fn from(err: ProgramError) -> Self {
        TradeBotErrors::ProgramErr(err)
    }
}


use thiserror::Error;
use num_enum::{FromPrimitive, IntoPrimitive};
use solana_program::program_error::ProgramError;

pub type TradeBotResult<T>  = Result<T, TradeBotError>;

#[derive(Error,Debug, PartialEq, Eq)]
pub enum TradeBotError {


    #[error("0:?")]
    Errors(#[from] TradeBotErrors),


    #[error(transparent)]
    ProgramError(ProgramError),


}

#[derive(Debug, IntoPrimitive, FromPrimitive, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TradeBotErrors {
    UnknownInstruction,
    InvalidInstruction,
    MarketAlreadyInitialized,
    MarketNotKnown,
    #[num_enum(default)]
    UnknownError
}


impl std::convert::From<TradeBotError> for ProgramError {
    fn from(e: TradeBotError) -> ProgramError {
        match e {
            TradeBotError::ProgramError(e) => e,
            TradeBotError::Errors(c) => ProgramError::Custom(c.into()),
        }
    }
}

impl From<ProgramError> for TradeBotError {
    fn from(err: ProgramError) -> Self {
        TradeBotError::ProgramError(err)
    }
}

impl std::fmt::Display for TradeBotErrors {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        <Self as std::fmt::Debug>::fmt(self, fmt)
    }
}

impl std::error::Error for TradeBotErrors {}
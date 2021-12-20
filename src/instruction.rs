use solana_program::msg;
use solana_program::pubkey::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};
use crate::error::{TradeBotResult, TradeBotError, TradeBotErrors};


#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum MarketStatus {
    UnInitialized = 0,
    Initialized = 1,
    Paused = 3,
    Stopped = 4,

}


#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct InitializeTradeMarket {
    pub address: Pubkey,
    pub trade_profit: u64,
    pub stopping_price: u64,
    pub starting_price_buy: u64,
    pub starting_price_sell: u64,
    pub simultaneous_open_positions: u64,
    pub status: MarketStatus,
    pub starting_base_balance: u64,
    pub starting_quote_balance: u64,
    pub starting_value: u64

}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct CloseTradeMarket {
    pub market_state: Pubkey,
}


#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Trade {
    buy_price: u64,
    sell_price: u64,
}





pub trait TradeBotInstruction {
    fn unpack(data: &[u8]) -> TradeBotResult<Box<Self>>;
}



impl TradeBotInstruction for InitializeTradeMarket {

    fn unpack(data: &[u8]) -> TradeBotResult<Box<Self>> {

        match InitializeTradeMarket::try_from_slice(data) {
            Ok(ix) => {
                msg!("InitializeTradeMarket Instruction");
                Ok(Box::new(ix))
            }
            Err(e) => {
                Err(TradeBotError::Errors(TradeBotErrors::InvalidInstruction))
            }
        }


    }
}

impl TradeBotInstruction for CloseTradeMarket {
    fn unpack(data: &[u8]) -> TradeBotResult<Box<Self>> {
        match CloseTradeMarket::try_from_slice(data) {
            Ok(ix) => {
                msg!("CloseTradeMarket Instruction {:?}, {:?}", ix, data);
                Ok(Box::new(ix))
            }
            Err(e) => {
                Err(TradeBotError::Errors(TradeBotErrors::InvalidInstruction))
            }
        }
    }
}
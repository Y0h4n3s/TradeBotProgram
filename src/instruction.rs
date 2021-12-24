use std::fmt::Debug;
use anchor_lang::AnchorDeserialize;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::msg;
use solana_program::pubkey::Pubkey;

use crate::error::{TradeBotError, TradeBotErrors, TradeBotResult};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum MarketStatus {
    UnInitialized = 0,
    Initialized = 1,
    Paused = 3,
    Stopped = 4,

}


#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct InitializeTradeMarket {


}


#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct RegisterTrader {
    pub register_date: u64
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct InitializeTrader {
    pub trade_profit: f64,
    pub stopping_price: f64,
    pub starting_price_buy: f64,
    pub starting_price_sell: f64,
    pub simultaneous_open_positions: u64,
    pub starting_base_balance: u64,
    pub starting_quote_balance: u64,
    pub starting_value: f64,
    pub serum_open_orders_rent: u64,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct DecommissionTrader {
    pub _padding: [u64; 7]
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct CloseTradeMarket {
    pub market_state: Pubkey,
    pub address: Pubkey,
}




#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Trade {
    pub buy_price: u128,
    pub sell_price: u128,
    pub size_base: u128,
    pub size_quote: u128,
    pub client_order_id: u128,
    pub _padding: [u8; 65]
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Settle {
    pub _padding: [u64; 10]
}

pub trait TradeBotInstruction<T: AnchorDeserialize  + Debug> {

    fn unpack(data: &[u8]) -> TradeBotResult<Box<T>> {
        match T::try_from_slice(data) {
            Ok(ix) => {
                msg!("{:?}", ix);
                Ok(Box::new(ix))
            }
            Err(e) => {
                Err(TradeBotErrors::InvalidInstruction)
            }
        }
    }
}




impl TradeBotInstruction<Self> for InitializeTradeMarket{

}

impl TradeBotInstruction<Self> for RegisterTrader{

}
impl TradeBotInstruction<Self> for Trade{

}

impl TradeBotInstruction<Self> for CloseTradeMarket{

}

impl TradeBotInstruction<Self> for InitializeTrader{

}

impl TradeBotInstruction<Self> for DecommissionTrader{

}

impl TradeBotInstruction<Self> for Settle{

}
// impl TradeBotInstruction for CloseTradeMarket {
//     fn unpack(data: &[u8]) -> TradeBotResult<Box<Self>> {
//         match CloseTradeMarket::try_from_slice(data) {
//             Ok(ix) => {
//                 msg!("{:?}", ix);
//                 Ok(Box::new(ix))
//             }
//             Err(e) => {
//                 Err(TradeBotError::Errors(TradeBotErrors::InvalidInstruction))
//             }
//         }
//     }
// }
//
// impl TradeBotInstruction for Trade {
//     fn unpack(data: &[u8]) -> TradeBotResult<Box<Self>> {
//         match Trade::try_from_slice(data) {
//             Ok(ix) => {
//                 msg!("{:?}", ix);
//                 Ok(Box::new(ix))
//             }
//             Err(e) => {
//                 Err(TradeBotError::Errors(TradeBotErrors::InvalidInstruction))
//             }
//         }
//     }
// }
//
// impl TradeBotInstruction for InitializeTrader {
//     fn unpack(data: &[u8]) -> TradeBotResult<Box<Self>> {
//         match InitializeTrader::try_from_slice(data) {
//             Ok(ix) => {
//                 msg!("{:?}", ix);
//                 Ok(Box::new(ix))
//             }
//             Err(e) => {
//                 Err(TradeBotError::Errors(TradeBotErrors::InvalidInstruction))
//             }
//         }
//     }
// }impl TradeBotInstruction for CloseTradeMarket {
//     fn unpack(data: &[u8]) -> TradeBotResult<Box<Self>> {
//         match CloseTradeMarket::try_from_slice(data) {
//             Ok(ix) => {
//                 msg!("{:?}", ix);
//                 Ok(Box::new(ix))
//             }
//             Err(e) => {
//                 Err(TradeBotError::Errors(TradeBotErrors::InvalidInstruction))
//             }
//         }
//     }
// }
//
// impl TradeBotInstruction for Trade {
//     fn unpack(data: &[u8]) -> TradeBotResult<Box<Self>> {
//         match Trade::try_from_slice(data) {
//             Ok(ix) => {
//                 msg!("{:?}", ix);
//                 Ok(Box::new(ix))
//             }
//             Err(e) => {
//                 Err(TradeBotError::Errors(TradeBotErrors::InvalidInstruction))
//             }
//         }
//     }
// }
//
// impl TradeBotInstruction for InitializeTrader {
//     fn unpack(data: &[u8]) -> TradeBotResult<Box<Self>> {
//         match InitializeTrader::try_from_slice(data) {
//             Ok(ix) => {
//                 msg!("{:?}", ix);
//                 Ok(Box::new(ix))
//             }
//             Err(e) => {
//                 Err(TradeBotError::Errors(TradeBotErrors::InvalidInstruction))
//             }
//         }
//     }
// }


use std::fmt::Debug;
use anchor_lang::{AnchorDeserialize, AnchorSerialize};
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
    pub trade_profit: u64,
    pub stopping_price: u64,
    pub starting_price_buy: u64,
    pub starting_price_sell: u64,
    pub simultaneous_open_positions: u64,
    pub starting_base_balance: u64,
    pub starting_quote_balance: u64,
    pub starting_value: u64,
    pub serum_open_orders_rent: u64,
    pub register_date: u64,
    pub padding: [u8; 16]
}


#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct UpdateTrader {
    pub trade_profit: u64,
    pub stopping_price: u64,
    pub simultaneous_open_positions: u64,
    pub _padding: [u8; 65]
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Deposit {
    pub base_amount: u64,
    pub quote_amount: u64,
    pub _padding: [u64; 7]
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
    pub _padding: [u8; 128]
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct Settle {
    pub _padding: [u64; 10]
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct CleanUp {
    pub _padding: [u64; 20]

}

pub trait TradeBotInstruction<T: AnchorSerialize + AnchorDeserialize  + Debug > {

    fn unpack(data: &[u8]) -> TradeBotResult<Box<T>> {
        match T::try_from_slice(data) {
            Ok(ix) => {
                Ok(Box::new(ix))
            }
            Err(e) => {
                Err(TradeBotErrors::InvalidInstruction)
            }
        }
    }
    fn pack(to_pack: T) -> TradeBotResult<Vec<u8>> {
        match to_pack.try_to_vec() {
            Ok(packed) => {
                Ok(packed)
            }
            Err(e) => {
                Err(TradeBotErrors::InvalidInstruction)
            }
        }
    }
}




impl TradeBotInstruction<Self> for CleanUp{

}

impl TradeBotInstruction<Self> for RegisterTrader{

}
impl TradeBotInstruction<Self> for Trade{

}

impl TradeBotInstruction<Self> for DecommissionTrader{

}

impl TradeBotInstruction<Self> for Settle{

}
impl TradeBotInstruction<Self> for UpdateTrader{

}
impl TradeBotInstruction<Self> for Deposit{

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


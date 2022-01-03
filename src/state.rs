use std::error::Error;

use anchor_lang::__private::bytemuck::{Pod, Zeroable};
use serum_dex::critbit::Slab;
use serum_dex::matching::Side;

use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::AccountInfo,
        msg,
        program_error::ProgramError,
        program_pack::{IsInitialized, Pack, Sealed},
        pubkey::Pubkey,
    },
};

use crate::error::TradeBotResult;
use crate::instruction::MarketStatus;

pub const TRADE_MARKET_STATE_SPAN: u64 = 129;
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TradeMarketState {
    pub serum_market_address: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub owner: Pubkey,
    pub status: MarketStatus,
}


impl Sealed for TradeMarketState {}

impl Pack for TradeMarketState {
    const LEN: usize = 129;
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut p = src;
        TradeMarketState::deserialize(&mut p).map_err(|_| {
            msg!("Failed to deserialize name record");
            ProgramError::InvalidAccountData
        })
    }
}

impl IsInitialized for TradeMarketState {
    fn is_initialized(&self) -> bool {
        self.status != MarketStatus::UnInitialized
    }
}


#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum TraderStatus {
    Registered,
    Initialized,
    Decommissioned,
    Stopped
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TraderState {
    pub market_address: Pubkey,
    pub base_trader_wallet: Pubkey,
    pub quote_trader_wallet: Pubkey,
    pub serum_open_orders: Pubkey,
    pub trader_signer: Pubkey,
    pub market_state: Pubkey,
    pub owner: Pubkey,
    pub min_trade_profit: u64,
    pub stopping_price: u64,
    pub starting_price_buy: u64,
    pub starting_price_sell: u64,
    pub simultaneous_open_positions: u64,
    pub starting_base_balance: u64,
    pub starting_quote_balance: u64,
    pub starting_value: u64,
    pub base_balance: u64,
    pub quote_balance: u64,
    pub value: u64,
    pub open_order_pairs: u64,
    pub total_txs: u64,
    pub register_date: u64,
    pub status: TraderStatus,
    pub _padding: [u64; 16]
}

pub const TRADER_SPAN: u64 = 465;



impl Sealed for TraderState {}

impl Pack for TraderState {
    const LEN: usize = 465;
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut p = src;
        TraderState::deserialize(&mut p).map_err(|_| {
            msg!("Failed to deserialize name record");
            ProgramError::InvalidAccountData
        })
    }
}

impl IsInitialized for TraderState {
    fn is_initialized(&self) -> bool {
        true
    }
}




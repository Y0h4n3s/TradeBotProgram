
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
use anchor_lang::__private::bytemuck::{Pod, Zeroable};
use serum_dex::matching::Side;
use serum_dex::critbit::Slab;
use std::error::Error;

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TradeMarketState {
    pub address: Pubkey,
    pub base_mint: Pubkey,
    pub base_market_wallet: Pubkey,
    pub quote_mint: Pubkey,
    pub quote_market_wallet: Pubkey,
    pub base_owner_account: Pubkey,
    pub quote_owner_account: Pubkey,
    pub market_signer: Pubkey,
    pub market_state: Pubkey,
    pub owner: Pubkey,
    pub open_orders: Pubkey,
    pub serum_open_orders: Pubkey,
    pub trade_profit: u64,
    pub stopping_price: u64,
    pub starting_price_buy: u64,
    pub starting_price_sell: u64,
    pub simultaneous_open_positions: u64,
    pub starting_base_balance: u64,
    pub starting_quote_balance: u64,
    pub starting_value: u64,
    pub status: MarketStatus
}
impl Sealed for TradeMarketState {}

impl Pack for TradeMarketState {
    const LEN: usize = 449;
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
pub struct OpenOrders {
    pub market_state: Pubkey,
    pub owner: Pubkey,
    pub orders: [u128; 65],
}

impl Copy for OpenOrders {}

impl IsInitialized for OpenOrders {
    fn is_initialized(&self) -> bool {
        true
    }

}
impl Sealed for OpenOrders {}

impl Pack for OpenOrders {
    const LEN: usize = 1104;
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut p = src;
        OpenOrders::deserialize(&mut p).map_err(|_| {
            msg!("Failed to deserialize name record");
            ProgramError::InvalidAccountData
        })
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct SerumOrderBook {
    pub side: Side,
    pub slab: Slab

}
impl SerumOrderBook {
    pub fn items(&self, descending: boolean) -> Result<(), Error> {
        if (self.slab)

        Ok(())
    }
}



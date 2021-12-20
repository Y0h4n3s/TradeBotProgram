use solana_program::{pubkey::Pubkey, entrypoint::ProgramResult};
use solana_program::account_info::{next_account_info, AccountInfo};
use crate::instruction::{TradeBotInstruction, InitializeTradeMarket, CloseTradeMarket, MarketStatus};
use solana_program::program_error::ProgramError;
use spl_token::instruction::transfer;
use anchor_lang::prelude::*;
use solana_program::msg;
use crate::error::TradeBotErrors::UnknownInstruction;
use crate::error::{TradeBotResult, TradeBotError, TradeBotErrors};
use solana_program::program::{invoke, invoke_signed};
use std::convert::TryFrom;
use crate::state::{TradeMarketState, OpenOrders};
use solana_program::program_pack::{Pack, IsInitialized};
use solana_program::program_memory::sol_memcmp;
use serum_dex::matching::{Side, OrderType};
use serum_dex::instruction::{SelfTradeBehavior, MarketInstruction};
use std::num::NonZeroU64;
use serum_dex::critbit::SlabView;
use anchor_lang::__private::bytemuck::__core::ops::{DerefMut, Deref};

pub struct Processor{}

impl Processor {
    pub fn process(program_id: &Pubkey,  accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        msg!("{:?}", data.len());
        match data.len() {
            97 => {
                let ix = InitializeTradeMarket::unpack(data).unwrap();
                Self::process_initialize_market(program_id,accounts, ix.as_ref());
                Ok(())
            }
            32 => {
                let ix = CloseTradeMarket::unpack(data).unwrap();
                Self::process_close_trade_market(program_id, accounts, ix.as_ref());
                Ok(())
            }
            0 => {
                Self::process_trigger_trade(program_id, accounts);
                Ok(())
            }
            _ => {
              Err(ProgramError::from(TradeBotError::Errors(UnknownInstruction)))
            }
        }
    }

    pub fn process_initialize_market(program_id: &Pubkey, accounts: &[AccountInfo], ix: &InitializeTradeMarket) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        let initializer = next_account_info(accounts_iter)?;
        let market_account = next_account_info(accounts_iter)?;
        let base_mint = next_account_info(accounts_iter)?;
        let quote_mint = next_account_info(accounts_iter)?;
        let base_market_wallet_account = next_account_info(accounts_iter)?;
        let quote_market_wallet_account = next_account_info(accounts_iter)?;
        let base_owner_account = next_account_info(accounts_iter)?;
        let quote_owner_account = next_account_info(accounts_iter)?;
        let open_orders_account = next_account_info(accounts_iter)?;
        let serum_open_orders_account = next_account_info(accounts_iter)?;
        let market = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;


        let market_account_seed = Self::calculate_seed_for_owner_and_market(market_account.key, initializer.key);
        let (pda, nonce) = Pubkey::find_program_address(&[market_account_seed.as_slice()], program_id);

        let owner_change_base_ix = spl_token::instruction::set_authority(
            token_program.key,
            base_market_wallet_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer.key,
            &[&initializer.key],
        ).unwrap();
        let owner_change_quote_ix = spl_token::instruction::set_authority(
            token_program.key,
            quote_market_wallet_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer.key,
            &[&initializer.key],
        ).unwrap();




        invoke(&owner_change_base_ix, accounts).unwrap();
        invoke(&owner_change_quote_ix, accounts).unwrap();



        let mut open_orders = OpenOrders::unpack(&mut open_orders_account.try_borrow_mut_data().unwrap()).unwrap();
        open_orders.owner = initializer.key.clone();
        open_orders.market_state = market_account.key.clone();
        OpenOrders::pack(open_orders.clone(), &mut open_orders_account.try_borrow_mut_data().unwrap()).unwrap();

        let new_market: TradeMarketState = TradeMarketState {
            address: ix.address.clone(),
            base_mint: base_mint.key.clone(),
            base_market_wallet: base_market_wallet_account.key.clone(),
            quote_mint: quote_mint.key.clone(),
            quote_market_wallet: quote_market_wallet_account.key.clone(),
            base_owner_account: base_owner_account.key.clone(),
            quote_owner_account: quote_owner_account.key.clone(),
            market_signer: pda.clone(),
            market_state: market_account.key.clone(),
            owner: initializer.key.clone(),
            open_orders: open_orders_account.key.clone(),
            serum_open_orders: serum_open_orders_account.key.clone(),
            trade_profit: ix.trade_profit,
            stopping_price: ix.stopping_price,
            starting_price_buy:  ix.starting_price_buy,
            starting_price_sell: ix.starting_price_sell ,
            simultaneous_open_positions: ix.simultaneous_open_positions,
            starting_base_balance: ix.starting_base_balance,
            starting_quote_balance: ix.starting_quote_balance,
            starting_value: ix.starting_value,
            status: MarketStatus::Initialized
        };

        TradeMarketState::pack(new_market.clone(), &mut market_account.try_borrow_mut_data().unwrap()).unwrap();


        msg!("{:?}", new_market);
        Ok(())
    }


    pub fn process_close_trade_market(program_id: &Pubkey, accounts: &[AccountInfo], ix: &CloseTradeMarket) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        let market_account = next_account_info(accounts_iter).unwrap();
        msg!("{:?} {:?}", market_account.key.to_string(), ix.market_state.to_string());
        if market_account.key.to_string() == ix.market_state.to_string() {
            let mut market_account_state = TradeMarketState::unpack(&mut market_account.try_borrow_mut_data().unwrap()).unwrap();
            if market_account_state.status == MarketStatus::UnInitialized {
                return Err(TradeBotError::Errors(TradeBotErrors::MarketNotKnown))
            }
            market_account_state.status = MarketStatus::Stopped;
            TradeMarketState::pack(market_account_state.clone(), &mut market_account.try_borrow_mut_data().unwrap()).unwrap();
            msg!("Market {:?} is closed", market_account.key.to_string())
        }

        Ok(())
    }

    pub fn process_trigger_trade(program_id: &Pubkey, accounts: &[AccountInfo]) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        let owner = next_account_info(accounts_iter).unwrap();
        let market_state = next_account_info(accounts_iter).unwrap();
        let base_market_wallet = next_account_info(accounts_iter).unwrap();
        let quote_market_wallet = next_account_info(accounts_iter).unwrap();
        let pda_info = next_account_info(accounts_iter).unwrap();
        let base_owner_account = next_account_info(accounts_iter).unwrap();
        let quote_owner_account = next_account_info(accounts_iter).unwrap();
        let market = next_account_info(accounts_iter).unwrap();
        let open_orders = next_account_info(accounts_iter).unwrap();
        let serum_open_orders = next_account_info(accounts_iter).unwrap();
        let request_queue = next_account_info(accounts_iter).unwrap();
        let event_queue = next_account_info(accounts_iter).unwrap();
        let bids_account = next_account_info(accounts_iter).unwrap();
        let asks_account = next_account_info(accounts_iter).unwrap();
        let coin_vault = next_account_info(accounts_iter).unwrap();
        let pc_vault = next_account_info(accounts_iter).unwrap();
        let token_program = next_account_info(accounts_iter).unwrap();
        let serum_program = next_account_info(accounts_iter).unwrap();
        let rent_sysvar = next_account_info(accounts_iter).unwrap();
        let mut market_account_state = TradeMarketState::unpack(&mut market_state.try_borrow_mut_data().unwrap()).unwrap();
        msg!("market state {:?}", market_account_state);
        let account = spl_token::state::Account::unpack(&mut base_market_wallet.try_borrow_mut_data().unwrap()).unwrap();
        let market_account_seed = Self::calculate_seed_for_owner_and_market(market_state.key, owner.key);
        let (pda, nonce) = Pubkey::find_program_address(&[market_account_seed.as_slice()], program_id);
        serum_dex::state::

        let new_order_ix = serum_dex::instruction::NewOrderInstructionV3 {
            side: Side::Ask,
            limit_price: NonZeroU64::new(1).unwrap(),
            max_coin_qty: NonZeroU64::new(1).unwrap(),
            max_native_pc_qty_including_fees: NonZeroU64::new(1).unwrap(),
            self_trade_behavior: SelfTradeBehavior::AbortTransaction,
            order_type: OrderType::Limit,
            client_order_id: 0,
            limit: 0
        };


        msg!("{:?} {:?}", bids.find_max().unwrap_or(0), asks.find_max().unwrap_or(0));
        let ix_data = MarketInstruction::NewOrderV3(new_order_ix).pack();
        let ix = solana_program::instruction::Instruction {
            program_id: serum_program.key.clone() ,
            accounts: vec![
                AccountMeta {pubkey: market.key.clone(), is_signer: false, is_writable: true },
                AccountMeta {pubkey: serum_open_orders.key.clone(), is_signer: false, is_writable: true},
                AccountMeta {pubkey: request_queue.key.clone(), is_signer: false, is_writable: true},
                AccountMeta {pubkey: event_queue.key.clone(), is_signer: false, is_writable: true},
                AccountMeta {pubkey: bids_account.key.clone(), is_signer: false, is_writable: true},
                AccountMeta {pubkey: asks_account.key.clone(), is_signer: false, is_writable: true},
                AccountMeta {pubkey: base_market_wallet.key.clone(), is_signer: false, is_writable: true},
                AccountMeta {pubkey: pda_info.key.clone(), is_signer: true, is_writable: true},
                AccountMeta {pubkey: coin_vault.key.clone(), is_signer: false, is_writable: true},
                AccountMeta {pubkey: pc_vault.key.clone(), is_signer: false, is_writable: true},
                AccountMeta {pubkey: token_program.key.clone(), is_signer: false, is_writable: false},
                AccountMeta {pubkey: rent_sysvar.key.clone(), is_signer: false, is_writable: false},
            ],
            data: ix_data
        };
        msg!("Sending Instruction");
        invoke_signed(
            &ix,
            &[
                market.clone(),
                serum_open_orders.clone(),
                request_queue.clone(),
                event_queue.clone(),
                bids_account.clone(),
                asks_account.clone(),
                base_market_wallet.clone(),
                pda_info.clone(),
                coin_vault.clone(),
                pc_vault.clone(),
                token_program.clone(),
                rent_sysvar.clone(),
                owner.clone()
            ],
            &[&[market_account_seed.as_slice(), &[nonce]]]).unwrap();
        Ok(())
    }

    fn calculate_seed_for_owner_and_market(owner: &Pubkey, market: &Pubkey) -> Vec<u8> {
        let mut i = 0;
        let owner_bytes = owner.to_bytes().clone();
        let market_bytes = market.to_bytes().clone();
        let mut seed: Vec<u8> = Vec::with_capacity(32);
        while i < owner.to_bytes().len() {
            seed.push(owner_bytes[i] + market_bytes[i]);
            i += 1
        }


        return seed
    }

}
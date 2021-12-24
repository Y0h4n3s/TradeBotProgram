use std::convert::TryFrom;
use std::f64;
use std::num::NonZeroU64;
use anchor_lang::__private::bytemuck::__core::ops::{Deref, DerefMut};
use anchor_lang::__private::bytemuck::bytes_of;
use anchor_lang::prelude::*;
use serum_dex::critbit::SlabView;
use serum_dex::instruction::{MarketInstruction, SelfTradeBehavior};
use serum_dex::matching::{OrderType, Side};
use serum_dex::state::Market;
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::msg;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::program_memory::sol_memcmp;
use solana_program::program_pack::{IsInitialized, Pack};
use spl_token::instruction::transfer;
use spl_token::state::Mint;

use crate::error::{TradeBotError, TradeBotErrors, TradeBotResult};
use crate::error::TradeBotErrors::UnknownInstruction;
use crate::instruction::{CloseTradeMarket, DecommissionTrader, InitializeTradeMarket, InitializeTrader, MarketStatus, RegisterTrader, Settle, Trade, TradeBotInstruction};
use crate::state::{TradeMarketState, TraderState, TraderStatus};

pub struct Processor {}

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        msg!("{:?}", data.len());
        match data.len() {
            0 => {
                let ix = InitializeTradeMarket::unpack(data).unwrap();
                Self::process_initialize_trade_market(program_id, accounts, ix.as_ref()).unwrap();
                Ok(())
            }
            72 => {
                let ix = InitializeTrader::unpack(data).unwrap();
                Self::process_initialize_trader(program_id, accounts, ix.as_ref()).unwrap();
                Ok(())
            }
            65 => {
                let ix = CloseTradeMarket::unpack(data).unwrap();
                Self::process_close_trade_market(program_id, accounts, ix.as_ref()).unwrap();
                Ok(())
            }
            145 => {
                let ix = Trade::unpack(data).unwrap();
                Self::process_trade(program_id, accounts, ix.as_ref()).unwrap();
                Ok(())
            }
            8 => {
                let ix = RegisterTrader::unpack(data).unwrap();
                Self::process_register_trader(program_id, accounts, ix.as_ref()).unwrap();
                Ok(())
            }
            56 => {
                let ix = DecommissionTrader::unpack(data).unwrap();
                Self::process_decommission_trader(program_id, accounts, ix.as_ref()).unwrap();
                Ok(())
            }
            80 => {
                let ix = Settle::unpack(data).unwrap();
                Self::process_settle(program_id, accounts, ix.as_ref()).unwrap();
                Ok(())
            }
            _ => {
                Err(ProgramError::from(UnknownInstruction))
            }
        }
    }

    pub fn process_initialize_trade_market(program_id: &Pubkey, accounts: &[AccountInfo], ix: &InitializeTradeMarket) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        let initializer = next_account_info(accounts_iter)?;
        let market_account = next_account_info(accounts_iter)?;
        let base_mint = next_account_info(accounts_iter)?;
        let quote_mint = next_account_info(accounts_iter)?;
        let market = next_account_info(accounts_iter)?;

        let new_market: TradeMarketState = TradeMarketState {
            serum_market_address: market.key.clone(),
            base_mint: base_mint.key.clone(),
            quote_mint: quote_mint.key.clone(),
            owner: initializer.key.clone(),
            status: MarketStatus::Initialized,
        };

        TradeMarketState::pack(new_market.clone(), &mut market_account.try_borrow_mut_data().unwrap()).unwrap();


        msg!("{:?}", new_market);
        Ok(())
    }

    pub fn process_register_trader(program_id: &Pubkey, accounts: &[AccountInfo], ix: &RegisterTrader) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        let initializer = next_account_info(accounts_iter)?;
        let market_state_account = next_account_info(accounts_iter)?;
        let trader_account = next_account_info(accounts_iter)?;

        let market_account_seed = Self::calculate_seed_for_owner_and_market(market_state_account.key, initializer.key);
        let (pda, nonce) = Pubkey::find_program_address(&[market_account_seed.as_slice()], program_id);

        let market_state = TradeMarketState::unpack(&mut market_state_account.try_borrow_mut_data().unwrap()).unwrap();


        let trader = TraderState {
            market_address: market_state.serum_market_address.clone(),
            base_market_wallet: pda,
            quote_market_wallet: pda,
            serum_open_orders: pda,
            market_signer: pda,
            market_state: market_state_account.key.clone(),
            owner: initializer.key.clone(),
            min_trade_profit: 0.0,
            stopping_price: 0.0,
            starting_price_buy: 0.0,
            starting_price_sell: 0.0,
            simultaneous_open_positions: 0,
            starting_base_balance: 0,
            starting_quote_balance: 0,
            starting_value: 0.0,
            total_txs: 0,
            status: TraderStatus::Registered
        };

        TraderState::pack(trader.clone(), &mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
        msg!("Registered Trader {:?}", trader);
        Ok(())

    }

    pub fn process_initialize_trader(program_id: &Pubkey, accounts: &[AccountInfo], ix: &InitializeTrader) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        let initializer = next_account_info(accounts_iter)?;
        let market_account = next_account_info(accounts_iter)?;
        let base_market_wallet_account = next_account_info(accounts_iter)?;
        let quote_market_wallet_account = next_account_info(accounts_iter)?;
        let trader_account = next_account_info(accounts_iter)?;
        let market_signer = next_account_info(accounts_iter)?;
        let serum_open_orders_account = next_account_info(accounts_iter)?;
        let initializer_base_wallet = next_account_info(accounts_iter)?;
        let initializer_quote_wallet = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;
        let serum_program = next_account_info(accounts_iter)?;

        let trader = TraderState::unpack(&mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
        if trader.status != TraderStatus::Registered {
            return Err(TradeBotErrors::TraderExists)
        }
        let market = TradeMarketState::unpack(&mut market_account.try_borrow_mut_data().unwrap()).unwrap();
        let market_account_seed = Self::calculate_seed_for_owner_and_market(market_account.key, initializer.key);
        let (pda, nonce) = Pubkey::find_program_address(&[market_account_seed.as_slice()], program_id);

        if pda.ne(market_signer.key) {
            return Err(TradeBotErrors::InvalidInstruction)
        }



        let initializer_base_account = spl_token::state::Account::unpack(&mut initializer_base_wallet.try_borrow_mut_data().unwrap()).unwrap();
        let initializer_quote_account = spl_token::state::Account::unpack(&mut initializer_quote_wallet.try_borrow_mut_data().unwrap()).unwrap();
        if initializer_base_account.amount < ix.starting_base_balance || initializer_quote_account.amount < ix.starting_quote_balance {
            return Err(TradeBotErrors::InsufficientTokens)
        }

        let transfer_base_to_market_wallet_ix = spl_token::instruction::transfer(
            token_program.key,
            initializer_base_wallet.key,
            base_market_wallet_account.key,
            initializer.key,
            &[],
            ix.starting_base_balance

        ).unwrap();

        let transfer_quote_to_market_wallet_ix = spl_token::instruction::transfer(
            token_program.key,
            initializer_quote_wallet.key,
            quote_market_wallet_account.key,
            initializer.key,
            &[],
            ix.starting_quote_balance

        ).unwrap();

        invoke(&transfer_base_to_market_wallet_ix, accounts).unwrap();
        invoke(&transfer_quote_to_market_wallet_ix, accounts).unwrap();
        let createSerumOpenOrdersAccountIx = solana_program::system_instruction::create_account(
            &pda,
            &serum_open_orders_account.key,
            ix.serum_open_orders_rent,
            3228,
            serum_program.key
        );
        invoke_signed(&createSerumOpenOrdersAccountIx, accounts, &[&[market_account_seed.as_slice(), &[nonce]]]).unwrap();

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

        let new_market: TraderState = TraderState {
            market_address: market.serum_market_address.clone(),
            base_market_wallet: base_market_wallet_account.key.clone(),
            quote_market_wallet: quote_market_wallet_account.key.clone(),
            serum_open_orders: serum_open_orders_account.key.clone(),
            market_signer: pda.clone(),
            market_state: market_account.key.clone(),
            owner: initializer.key.clone(),
            min_trade_profit: ix.trade_profit,
            stopping_price: ix.stopping_price,
            starting_price_buy: ix.starting_price_buy,
            starting_price_sell: ix.starting_price_sell,
            simultaneous_open_positions: ix.simultaneous_open_positions,
            starting_base_balance: ix.starting_base_balance,
            starting_quote_balance: ix.starting_quote_balance,
            starting_value: ix.starting_value,
            total_txs: 0,
            status: TraderStatus::Initialized
        };

        TraderState::pack(new_market.clone(), &mut trader_account.try_borrow_mut_data().unwrap()).unwrap();


        msg!("{:?}", new_market);
        Ok(())
    }

    pub fn process_decommission_trader(program_id: &Pubkey, accounts: &[AccountInfo], ix: &DecommissionTrader) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        let initializer = next_account_info(accounts_iter).unwrap();
        let trader_account = next_account_info(accounts_iter).unwrap();
        let mut trader = TraderState::unpack(&mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
        if trader.owner.ne(initializer.key) {
           return Err(TradeBotErrors::Unauthorized)
        }
        trader.status = TraderStatus::Decommissioned;
        TraderState::pack(trader.clone(), &mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
        msg!("Trader Decommissioned");

        Ok(())
    }


    pub fn process_close_trade_market(program_id: &Pubkey, accounts: &[AccountInfo], ix: &CloseTradeMarket) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        let market_account = next_account_info(accounts_iter).unwrap();
        msg!("{:?} {:?}", market_account.key.to_string(), ix.market_state.to_string());
        if market_account.key.to_string() == ix.market_state.to_string() {
            let mut market_account_state = TradeMarketState::unpack(&mut market_account.try_borrow_mut_data().unwrap()).unwrap();
            if market_account_state.status == MarketStatus::UnInitialized {
                return Err(TradeBotErrors::MarketNotKnown);
            }
            market_account_state.status = MarketStatus::Stopped;
            TradeMarketState::pack(market_account_state.clone(), &mut market_account.try_borrow_mut_data().unwrap()).unwrap();
            msg!("Market {:?} is closed", market_account.key.to_string())
        }

        Ok(())
    }

    pub fn process_trade(program_id: &Pubkey, accounts: &[AccountInfo], ix: &Trade) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        //let owner = next_account_info(accounts_iter).unwrap();
        let base_market_wallet = next_account_info(accounts_iter).unwrap();
        let quote_market_wallet = next_account_info(accounts_iter).unwrap();
        let market_signer = next_account_info(accounts_iter).unwrap();
        let market = next_account_info(accounts_iter).unwrap();
        let request_queue = next_account_info(accounts_iter).unwrap();
        let event_queue = next_account_info(accounts_iter).unwrap();
        let bids_account = next_account_info(accounts_iter).unwrap();
        let asks_account = next_account_info(accounts_iter).unwrap();
        let coin_vault = next_account_info(accounts_iter).unwrap();
        let pc_vault = next_account_info(accounts_iter).unwrap();
        let trader_account = next_account_info(accounts_iter).unwrap();
        let serum_open_orders_account = next_account_info(accounts_iter).unwrap();
        let token_program = next_account_info(accounts_iter).unwrap();
        let serum_program = next_account_info(accounts_iter).unwrap();
        let rent_sysvar = next_account_info(accounts_iter).unwrap();
        let mut trader = TraderState::unpack(&mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
        let account = spl_token::state::Account::unpack(&mut base_market_wallet.try_borrow_mut_data().unwrap()).unwrap();
        let market_account_seed = Self::calculate_seed_for_owner_and_market(&trader.market_state, &trader.owner);
        let (pda, nonce) = Pubkey::find_program_address(&[market_account_seed.as_slice()], program_id);

        //msg!("{} {} {}", sell_price.clone(), max_base_quantity.clone(), max_quote_quantity.clone());
        let new_order_sell_ix = serum_dex::instruction::NewOrderInstructionV3 {
            side: Side::Ask,
            limit_price: NonZeroU64::new(ix.sell_price as u64).unwrap(),
            max_coin_qty: NonZeroU64::new(ix.size_base as u64).unwrap(),
            max_native_pc_qty_including_fees: NonZeroU64::new(1_000_000_000_000).unwrap(),
            self_trade_behavior: SelfTradeBehavior::AbortTransaction,
            order_type: OrderType::Limit,
            client_order_id: ix.client_order_id as u64,
            limit: 0,
        };


        msg!("");
        let new_order_buy_ix = serum_dex::instruction::NewOrderInstructionV3 {
            side: Side::Bid,
            limit_price: NonZeroU64::new(ix.buy_price as u64).unwrap(),
            max_coin_qty: NonZeroU64::new(ix.size_quote as u64).unwrap(),
            max_native_pc_qty_including_fees:NonZeroU64::new(1_000_000_000_000).unwrap(),
            self_trade_behavior: SelfTradeBehavior::AbortTransaction,
            order_type: OrderType::Limit,
            client_order_id: ix.client_order_id as u64,
            limit: 0,
        };


        let ix_data_sell = MarketInstruction::NewOrderV3(new_order_sell_ix).pack();
        let ix_data_buy = MarketInstruction::NewOrderV3(new_order_buy_ix).pack();
        let ix_sell = solana_program::instruction::Instruction {
            program_id: serum_program.key.clone(),
            accounts: vec![
                AccountMeta { pubkey: market.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: serum_open_orders_account.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: request_queue.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: event_queue.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: bids_account.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: asks_account.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: base_market_wallet.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: market_signer.key.clone(), is_signer: true, is_writable: true },
                AccountMeta { pubkey: coin_vault.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: pc_vault.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: token_program.key.clone(), is_signer: false, is_writable: false },
                AccountMeta { pubkey: rent_sysvar.key.clone(), is_signer: false, is_writable: false },
            ],
            data: ix_data_sell,
        };
        let ix_buy = solana_program::instruction::Instruction {
            program_id: serum_program.key.clone(),
            accounts: vec![
                AccountMeta { pubkey: market.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: serum_open_orders_account.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: request_queue.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: event_queue.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: bids_account.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: asks_account.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: quote_market_wallet.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: market_signer.key.clone(), is_signer: true, is_writable: true },
                AccountMeta { pubkey: coin_vault.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: pc_vault.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: token_program.key.clone(), is_signer: false, is_writable: false },
                AccountMeta { pubkey: rent_sysvar.key.clone(), is_signer: false, is_writable: false },
            ],
            data: ix_data_buy,
        };
        msg!("Sending Instructions");
        invoke_signed(
            &ix_buy,
            accounts,
            &[&[market_account_seed.as_slice(), &[nonce]]]).unwrap();
        invoke_signed(
            &ix_sell,
            accounts,
            &[&[market_account_seed.as_slice(), &[nonce]]]).unwrap();

        trader.total_txs += 2;
        TraderState::pack(trader, &mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
        msg!("Orders are placed");
        Ok(())
    }

    pub fn process_settle(program_id: &Pubkey, accounts: &[AccountInfo], ix: &Settle) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        let trader_account = next_account_info(accounts_iter).unwrap();
        let market = next_account_info(accounts_iter).unwrap();
        let serum_open_orders = next_account_info(accounts_iter).unwrap();
        let market_signer = next_account_info(accounts_iter).unwrap();
        let coin_vault = next_account_info(accounts_iter).unwrap();
        let pc_vault = next_account_info(accounts_iter).unwrap();
        let base_market_wallet = next_account_info(accounts_iter).unwrap();
        let quote_market_wallet = next_account_info(accounts_iter).unwrap();
        let vault_signer = next_account_info(accounts_iter).unwrap();
        let serum_program = next_account_info(accounts_iter).unwrap();
        let token_program = next_account_info(accounts_iter).unwrap();
        let trader = TraderState::unpack(&mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
        let market_account_seed = Self::calculate_seed_for_owner_and_market(&trader.market_state, &trader.owner);
        let (pda, nonce) = Pubkey::find_program_address(&[market_account_seed.as_slice()], program_id);
        let base_wallet = spl_token::state::Account::unpack(&mut base_market_wallet.try_borrow_mut_data().unwrap()).unwrap();
        let quote_wallet = spl_token::state::Account::unpack(&mut quote_market_wallet.try_borrow_mut_data().unwrap()).unwrap();

        msg!("before {:?} {:?}", base_wallet, quote_wallet);
        let data = MarketInstruction::SettleFunds.pack();
        let mut accounts1: Vec<AccountMeta> = vec![
            AccountMeta::new(market.key.clone(), false),
            AccountMeta::new(serum_open_orders.key.clone(), false),
            AccountMeta::new(market_signer.key.clone(), true),
            AccountMeta::new(coin_vault.key.clone(), false),
            AccountMeta::new(pc_vault.key.clone(), false),
            AccountMeta::new(base_market_wallet.key.clone(), false),
            AccountMeta::new(quote_market_wallet.key.clone(), false),
            AccountMeta::new_readonly(vault_signer.key.clone(), false),
            AccountMeta::new_readonly(token_program.key.clone(), false),
        ];

        let ix = solana_program::instruction::Instruction {
            program_id: serum_program.key.clone(),
            accounts: accounts1,
            data
        };



        invoke_signed(&ix, accounts, &[&[market_account_seed.as_slice(), &[nonce]]]).unwrap();
        msg!("after {:?} {:?}", base_wallet, quote_wallet);
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


        return seed;
    }

    fn price_number_to_lots(market: &Market, price: f64, base_mint: &Mint, quote_mint: &Mint) -> u64 {
        return ((price * 10_f64.powf(f64::from(quote_mint.decimals))) as u64 * market.coin_lot_size) / (10_u64.pow(u32::from(base_mint.decimals)) * market.pc_lot_size)
    }

    fn base_size_number_to_lots(market: &Market, size: f64, base_mint: &Mint) -> u64 {
        return (size * 10_f64.powf(f64::from(base_mint.decimals))) as u64 / market.coin_lot_size
    }
}
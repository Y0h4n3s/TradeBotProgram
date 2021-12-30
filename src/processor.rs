use std::borrow::Borrow;
use std::cell::RefMut;
use std::cmp::{max, min, Reverse};
use std::convert::{TryFrom, TryInto};
use std::f64;
use std::num::NonZeroU64;
use anchor_lang::__private::bytemuck::__core::ops::{Deref, DerefMut};
use anchor_lang::__private::bytemuck::bytes_of;
use anchor_lang::prelude::*;
use serum_dex::critbit::{Slab, SlabView};
use serum_dex::instruction::{MarketInstruction, SelfTradeBehavior};
use serum_dex::matching::{OrderType, Side};
use serum_dex::state::{Market, ToAlignedBytes};
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::log::sol_log_compute_units;
use solana_program::msg;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::program_memory::sol_memcmp;
use solana_program::program_pack::{IsInitialized, Pack};
use spl_token::instruction::transfer;
use spl_token::state::Mint;

use crate::error::{TradeBotError, TradeBotErrors, TradeBotResult};
use crate::error::TradeBotErrors::UnknownInstruction;
use crate::instruction::{CloseTradeMarket, DecommissionTrader, InitializeTradeMarket, MarketStatus, RegisterTrader, Settle, Trade, TradeBotInstruction, UpdateTrader};
use crate::state::{TradeMarketState, TraderState, TraderStatus};


pub const TRADES_CLIENT_ID: u64 = 1221144433222;
pub struct Processor {}

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> TradeBotResult<()> {
        match data.len() {
            0 => {
                let ix = InitializeTradeMarket::unpack(data).unwrap();
                Self::process_initialize_trade_market(program_id, accounts, ix.as_ref())
            }
            65 => {
                let ix = CloseTradeMarket::unpack(data).unwrap();
                Self::process_close_trade_market(program_id, accounts, ix.as_ref())
            }
            128 => {
                let ix = Trade::unpack(data).unwrap();
                Self::process_trade(program_id, accounts, ix.as_ref())

            }
            96 => {
                let ix = RegisterTrader::unpack(data).unwrap();
                Self::process_register_trader(program_id, accounts, ix.as_ref())
            }
            56 => {
                let ix = DecommissionTrader::unpack(data).unwrap();
                Self::process_decommission_trader(program_id, accounts, ix.as_ref())
            }
            80 => {
                let ix = Settle::unpack(data).unwrap();
                Self::process_settle(program_id, accounts, ix.as_ref())
            }
            89 => {
                let ix = UpdateTrader::unpack(data).unwrap();
                Self::process_update_trader(program_id, accounts, ix.as_ref())

            }
            _ => {
                Err(TradeBotErrors::UnknownInstruction)
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
        let base_market_wallet_account = next_account_info(accounts_iter)?;
        let quote_market_wallet_account = next_account_info(accounts_iter)?;
        let trader_account = next_account_info(accounts_iter)?;
        let trader_signer = next_account_info(accounts_iter)?;
        let serum_open_orders_account = next_account_info(accounts_iter)?;
        let initializer_base_wallet = next_account_info(accounts_iter)?;
        let initializer_quote_wallet = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;
        let serum_program = next_account_info(accounts_iter)?;



        let market_state = TradeMarketState::unpack(&mut market_state_account.try_borrow_mut_data().unwrap()).unwrap();
        let market_account_seed = Self::calculate_seed_for_owner_and_market(&market_state.serum_market_address.clone(), initializer.key);
        let (pda, nonce) = Pubkey::find_program_address(&[market_account_seed.as_slice()], program_id);

        if pda.ne(trader_signer.key) {
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
            ix.starting_quote_balance as u64

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


        let trader = TraderState {
            market_address: market_state.serum_market_address.clone(),
            base_trader_wallet: base_market_wallet_account.key.clone(),
            quote_trader_wallet: quote_market_wallet_account.key.clone(),
            serum_open_orders: serum_open_orders_account.key.clone(),
            trader_signer: pda.clone(),
            market_state: market_state_account.key.clone(),
            owner: initializer.key.clone(),
            min_trade_profit: ix.trade_profit,
            stopping_price: ix.stopping_price,
            starting_price_buy: ix.starting_price_buy,
            starting_price_sell: ix.starting_price_sell,
            simultaneous_open_positions: ix.simultaneous_open_positions,
            starting_base_balance: ix.starting_base_balance,
            starting_quote_balance: ix.starting_quote_balance,
            starting_value: ix.starting_value,
            base_balance: ix.starting_base_balance,
            quote_balance: ix.starting_quote_balance,
            value: ix.starting_value,
            open_order_pairs: 0,
            total_txs: 0,
            register_date: ix.register_date,
            status: TraderStatus::Registered,
            _padding: vec![0 as u64; 16].try_into().unwrap()
        };

        TraderState::pack(trader.clone(), &mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
        msg!("Registered Trader {:?}", trader);
        Ok(())

    }



    pub fn process_update_trader(program_id: &Pubkey, accounts: &[AccountInfo], ix: &UpdateTrader) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        let initializer = next_account_info(accounts_iter).unwrap();
        let trader_account = next_account_info(accounts_iter).unwrap();

        if !initializer.is_signer  {
            return Err(TradeBotErrors::Unauthorized)
        }


        let mut trader = TraderState::unpack(&mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
        if trader.owner.ne(initializer.key) {
           return Err(TradeBotErrors::Unauthorized)
        }
        trader.stopping_price = ix.stopping_price;
        trader.simultaneous_open_positions = ix.simultaneous_open_positions;
        trader.min_trade_profit = ix.trade_profit;
        TraderState::pack(trader.clone(), &mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
        msg!("Trader Updated {:?}", trader);

        Ok(())
    }

    pub fn process_decommission_trader(program_id: &Pubkey, accounts: &[AccountInfo], ix: &DecommissionTrader) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        let initializer = next_account_info(accounts_iter).unwrap();
        let trader_account = next_account_info(accounts_iter).unwrap();

        if trader_account.owner.ne(initializer.key) {
           return Err(TradeBotErrors::Unauthorized)
        }
        let mut trader = TraderState::unpack(&mut trader_account.try_borrow_mut_data().unwrap()).unwrap();

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

    pub fn process_cleanup_decommissioned_trader(program_id: &Pubkey, accounts: &[AccountInfo], ix: &CloseTradeMarket) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        let serum_market_account = next_account_info(accounts_iter).unwrap();
        let serum_open_orders_account = next_account_info(accounts_iter).unwrap();
        let bids_account = next_account_info(accounts_iter).unwrap();
        let asks_account = next_account_info(accounts_iter).unwrap();
        let event_queue = next_account_info(accounts_iter).unwrap();
        let market_state_account = next_account_info(accounts_iter).unwrap();
        let trader_account = next_account_info(accounts_iter).unwrap();
        let trader_signer_account = next_account_info(accounts_iter).unwrap();
        let serum_program = next_account_info(accounts_iter).unwrap();



        let serum_market_account_clone = serum_market_account.clone();
        let serum_open_orders_account_clone = serum_open_orders_account.clone();
        let asks_clone = asks_account.clone();
        let bids_clone = bids_account.clone();
        let mut trader = TraderState::unpack(&mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
        let serum_market = serum_dex::state::Market::load(serum_market_account, serum_program.key, true).unwrap();
        let bids_slab = serum_market.load_bids_mut(bids_account).unwrap();
        let asks_slab = serum_market.load_asks_mut(asks_account).unwrap();
        let mut all_bids = Self::parse_order_book(Side::Bid, bids_slab.deref().clone());
        let mut all_asks = Self::parse_order_book(Side::Ask, asks_slab.deref().clone());
        let mut my_orders: Vec<Order> = vec![];
        let serum_open_order_as = serum_open_orders_account.key.clone().to_aligned_bytes();

        all_bids.iter().for_each(|order| {
            if order.owner == serum_open_order_as {
                my_orders.push(order.clone())
            }
        });
        all_asks.iter().for_each(|order| {
            my_orders.push(order.clone())
        });

        msg!("Num of orders: {}", my_orders.len());

        let market_state = TradeMarketState::unpack(&mut market_state_account.try_borrow_mut_data().unwrap()).unwrap();
        let market_account_seed = Self::calculate_seed_for_owner_and_market(&market_state.serum_market_address.clone(), &trader.owner);
        let (pda, nonce) = Pubkey::find_program_address(&[market_account_seed.as_slice()], program_id);

        for i in 0..min(10, my_orders.len()) {
            let or = my_orders.get(i).unwrap();
            let cancel_order_ix = serum_dex::instruction::CancelOrderInstructionV2 {
                side: or.side,
                order_id: or.order_id
            };
            let cancel_order_ix_packed = MarketInstruction::CancelOrderV2(cancel_order_ix).pack();
            let ix = solana_program::instruction::Instruction {
                data: cancel_order_ix_packed,
                accounts: vec![
                    AccountMeta::new(serum_market_account_clone.key.clone(), false),
                    AccountMeta::new(bids_clone.key.clone(), false),
                    AccountMeta::new(asks_clone.key.clone(), false),
                    AccountMeta::new(serum_open_orders_account_clone.key.clone(), false),
                    AccountMeta::new_readonly(trader.trader_signer, true),
                    AccountMeta::new(event_queue.key.clone(), false)
                ],
                program_id: serum_program.key.clone()
            };
            invoke_signed(&ix, &[
                serum_open_orders_account_clone.clone(),
                bids_clone.clone(),
                asks_clone.clone(),
                serum_open_orders_account_clone.clone(),
                trader_signer_account.clone(),
                event_queue.clone()
            ],
            &[&[market_account_seed.as_slice(), &[nonce]]]).unwrap()
        }
        Ok(())

    }
    pub fn process_trade(program_id: &Pubkey, accounts: &[AccountInfo], ix: &Trade) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        let base_market_wallet = next_account_info(accounts_iter).unwrap();
        let quote_market_wallet = next_account_info(accounts_iter).unwrap();
        let trader_signer = next_account_info(accounts_iter).unwrap();
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
        let market_account_seed = Self::calculate_seed_for_owner_and_market(&trader.market_address, &trader.owner);
        let (pda, nonce) = Pubkey::find_program_address(&[market_account_seed.as_slice()], program_id);
        let serum_open_order_as = serum_open_orders_account.key.clone().to_aligned_bytes();


        if pda.ne(trader_signer.key) {
            return Err(TradeBotErrors::Unauthorized)
        }

        let base_account = spl_token::state::Account::unpack(&base_market_wallet.try_borrow_data().unwrap()).unwrap();
        let quote_account = spl_token::state::Account::unpack(&quote_market_wallet.try_borrow_data().unwrap()).unwrap();
        let market_clone = market.clone();
        let serum_open_orders_clone = serum_open_orders_account.clone();
        let asks_clone = asks_account.clone();
        let bids_clone = bids_account.clone();
        let mut trader = TraderState::unpack(&mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
        let serum_market = serum_dex::state::Market::load(market, serum_program.key, true).unwrap();
        let bids_slab = serum_market.load_bids_mut(bids_account).unwrap();
        let asks_slab = serum_market.load_asks_mut(asks_account).unwrap();
        let mut all_bids = Self::parse_order_book(Side::Bid, bids_slab.deref().clone());
        let mut all_asks = Self::parse_order_book(Side::Ask, asks_slab.deref().clone());

        if all_bids.len() < 1 || all_asks.len() < 1{
            return Err(TradeBotErrors::NoTradesFoundOnMarket)
        }


        all_bids.sort_by_key(|k| Reverse(k.price));
        all_asks.sort_by_key(|k| k.price);
        let buy_price = all_bids.get(0).unwrap().price;
        let sell_price = all_asks.get(0).unwrap().price;
        let market_price = (buy_price + sell_price) / 2;
        let base_size = trader.base_balance  / (trader.simultaneous_open_positions - trader.open_order_pairs * 2);

        let quote_size = trader.quote_balance / (trader.simultaneous_open_positions - trader.open_order_pairs * 2);
        let mut price_gap_buy = market_price - ((quote_size * market_price) / (trader.min_trade_profit / 2 + quote_size));
        let mut price_gap_sell = ((trader.min_trade_profit + 2 * base_size * market_price) / (2 * base_size)) - market_price;
        if price_gap_buy == 0 {
            price_gap_buy = 1
        }
        if price_gap_sell == 0 {
            price_gap_sell = 1
        }
        let order_buy_price = market_price - price_gap_buy;
        let order_sell_price = market_price + price_gap_sell;
        msg!("{} {}", price_gap_buy, price_gap_sell);
        if base_account.amount  <= base_size || quote_account.amount <= quote_size {
            return Err(TradeBotErrors::InsufficientTokens)
        }

        if trader.open_order_pairs >= trader.simultaneous_open_positions / 2 {
            return Err(TradeBotErrors::ExceededOpenOrdersLimit)
        }

        if trader.stopping_price  >= buy_price || trader.stopping_price  >= sell_price {
            return Err(TradeBotErrors::StopLossLimit)
        }

        let mut open_prices: Vec<u64> = vec![];
        all_bids.iter().for_each(|order| {
            if order.owner == serum_open_order_as {
                open_prices.push(order.price);
                open_prices.push(order.client_id);
            }
        });
        all_asks.iter().for_each(|order| {
            if order.owner == serum_open_order_as {
                open_prices.push(order.price);
                open_prices.push(order.client_id);
            }
        });

        for open_price in open_prices {
            if open_price >= order_buy_price && open_price <= order_sell_price {
                return Err(TradeBotErrors::PriceAlreadyTraded)
            }
        }

        let base_size_lots = base_size/serum_market.coin_lot_size;
        let quote_size_lots = (quote_size / order_buy_price);
        msg!("{:?} {:?} {:?} ", base_size_lots, base_size, quote_size_lots);
        msg!("{:?} {:?} {:?} ", order_buy_price, order_sell_price, market_price);
        let new_order_sell_ix = serum_dex::instruction::NewOrderInstructionV3 {
            side: Side::Ask,
            limit_price: NonZeroU64::new(order_sell_price).unwrap(),
            max_coin_qty: NonZeroU64::new(base_size_lots).unwrap(),
            max_native_pc_qty_including_fees: NonZeroU64::new(1).unwrap(),
            self_trade_behavior: SelfTradeBehavior::AbortTransaction,
            order_type: OrderType::Limit,
            client_order_id: order_buy_price,
            limit: 0,
        };


        let new_order_buy_ix = serum_dex::instruction::NewOrderInstructionV3 {
            side: Side::Bid,
            limit_price: NonZeroU64::new(order_buy_price).unwrap(),
            max_coin_qty: NonZeroU64::new(quote_size).unwrap(),
            max_native_pc_qty_including_fees:NonZeroU64::new(quote_size).unwrap(),
            self_trade_behavior: SelfTradeBehavior::AbortTransaction,
            order_type: OrderType::Limit,
            client_order_id: order_sell_price,
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
                AccountMeta { pubkey: trader_signer.key.clone(), is_signer: true, is_writable: false },
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
                AccountMeta { pubkey: trader_signer.key.clone(), is_signer: true, is_writable: false },
                AccountMeta { pubkey: coin_vault.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: pc_vault.key.clone(), is_signer: false, is_writable: true },
                AccountMeta { pubkey: token_program.key.clone(), is_signer: false, is_writable: false },
                AccountMeta { pubkey: rent_sysvar.key.clone(), is_signer: false, is_writable: false },
            ],
            data: ix_data_buy,
        };
        std::mem::drop(serum_market);
        std::mem::drop(bids_slab);
        std::mem::drop(asks_slab);
        msg!("Sending Instructions");
        invoke_signed(
            &ix_buy,
            &[
                market_clone.clone(),
                serum_open_orders_clone.clone(),
                request_queue.clone(),
                event_queue.clone(),
                bids_clone.clone(),
                asks_clone.clone(),
                quote_market_wallet.clone(),
                trader_signer.clone(),
                coin_vault.clone(),
                pc_vault.clone(),
                token_program.clone(),
                rent_sysvar.clone()
            ],
            &[&[market_account_seed.as_slice(), &[nonce]]]).unwrap();
        invoke_signed(
            &ix_sell,
            &[
                market_clone,
                serum_open_orders_clone,
                request_queue.clone(),
                event_queue.clone(),
                bids_clone,
                asks_clone,
                base_market_wallet.clone(),
                trader_signer.clone(),
                coin_vault.clone(),
                pc_vault.clone(),
                token_program.clone(),
                rent_sysvar.clone()
            ],
            &[&[market_account_seed.as_slice(), &[nonce]]]).unwrap();

        trader.base_balance -= base_size;
        trader.quote_balance -= quote_size;
        trader.open_order_pairs += 1;
        trader.total_txs += 2;
        trader.status = TraderStatus::Initialized;
        TraderState::pack(trader, &mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
        msg!("Orders are placed");
        Ok(())
    }

    pub fn process_settle(program_id: &Pubkey, accounts: &[AccountInfo], ix: &Settle) -> TradeBotResult<()> {
        let accounts_iter = &mut accounts.iter();
        let trader_account = next_account_info(accounts_iter).unwrap();
        let market = next_account_info(accounts_iter).unwrap();
        let serum_open_orders = next_account_info(accounts_iter).unwrap();
        let bids = next_account_info(accounts_iter).unwrap();
        let asks = next_account_info(accounts_iter).unwrap();
        let trader_signer = next_account_info(accounts_iter).unwrap();
        let coin_vault = next_account_info(accounts_iter).unwrap();
        let pc_vault = next_account_info(accounts_iter).unwrap();
        let base_market_wallet = next_account_info(accounts_iter).unwrap();
        let quote_market_wallet = next_account_info(accounts_iter).unwrap();
        let vault_signer = next_account_info(accounts_iter).unwrap();
        let serum_program = next_account_info(accounts_iter).unwrap();
        let token_program = next_account_info(accounts_iter).unwrap();
        let market_clone = market.clone();
        let serum_open_orders_clone = serum_open_orders.clone();
        let mut trader = TraderState::unpack(&mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
        let market_account_seed = Self::calculate_seed_for_owner_and_market(&trader.market_address, &trader.owner);
        let (pda, nonce) = Pubkey::find_program_address(&[market_account_seed.as_slice()], program_id);
        if pda.to_string() != trader_signer.key.to_string() {
            return Err(TradeBotErrors::Unauthorized)
        }

        let serum_open_order_as = serum_open_orders.key.clone().to_aligned_bytes();

        let data = MarketInstruction::SettleFunds.pack();
        msg!("{:?}", serum_program.key.to_string());
        let mut accounts1: Vec<AccountMeta> = vec![
            AccountMeta::new(market.key.clone(), false),
            AccountMeta::new(serum_open_orders.key.clone(), false),
            AccountMeta::new_readonly(trader_signer.key.clone(), true),
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

        let serum_market = serum_dex::state::Market::load(market, serum_program.key, true).unwrap();
        let bids = Self::parse_order_book_for_owner(Side::Bid, serum_market.load_bids_mut(bids).unwrap().deref().clone(), &serum_open_order_as);
        let asks = Self::parse_order_book_for_owner(Side::Ask, serum_market.load_asks_mut(asks).unwrap().deref().clone(), &serum_open_order_as);
        let open_orders = serum_market.load_orders_mut(serum_open_orders, Some(trader_signer), serum_program.key, None, None).unwrap();
        // msg!("{:?} {:?} {:?}", open_orders.native_coin_total, open_orders.native_pc_free, open_orders.native_coin_free);serum_market.borrow();
        // msg!("{:?} {:?} {:?}", open_orders.native_pc_total, bids, asks);


        trader.base_balance += open_orders.native_coin_free;
        trader.quote_balance += open_orders.native_pc_free;
        trader.open_order_pairs = max(bids.len() as u64, asks.len() as u64);

        std::mem::drop(serum_market);
        std::mem::drop(open_orders);
        invoke_signed(&ix, &[
            market_clone,
            serum_open_orders_clone,
            trader_signer.clone(),
            coin_vault.clone(),
            pc_vault.clone(),
            base_market_wallet.clone(),
            quote_market_wallet.clone(),
            vault_signer.clone(),
            token_program.clone()
        ], &[&[market_account_seed.as_slice(), &[nonce]]]).unwrap();

        TraderState::pack(trader, &mut trader_account.try_borrow_mut_data().unwrap()).unwrap();
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

    fn parse_order_book_for_owner(side: Side, slab: &Slab, owner: &[u64; 4]) -> Vec<Order>{
        let mut filtered: Vec<Order> = vec![];
        for i in 0 .. slab.capacity() {
            match slab.get(i as u32) {
                Some(node) => {
                    match node.as_leaf() {
                        Some(n) => {
                            if  &n.owner() == owner {
                                filtered.push(Order {
                                    side,
                                    price: u64::try_from(n.price()).unwrap(),
                                    size: n.quantity(),
                                    client_id: n.client_order_id(),
                                    owner: n.owner(),
                                    order_id: n.order_id()
                                })
                            }

                        }
                        None => {}
                    }
                }
                None => {}
            }
        }
        filtered
    }

    fn parse_order_book(side: Side, slab: &Slab) -> Vec<Order>{
        let mut filtered: Vec<Order> = vec![];
        for i in 0 .. slab.capacity() {
            match slab.get(i as u32) {
                Some(node) => {
                    match node.as_leaf() {
                        Some(n) => {

                                filtered.push(Order {
                                    side,
                                    price: u64::try_from(n.price()).unwrap(),
                                    size: n.quantity(),
                                    client_id: n.client_order_id(),
                                    owner: n.owner(),
                                    order_id: n.order_id()

                                })


                        }
                        None => {}
                    }
                }
                None => {}
            }
        }
        filtered
    }



}
#[derive(Debug, Clone, Copy)]
pub struct Order {
    side: Side,
    price: u64,
    size: u64,
    client_id: u64,
    owner: [u64; 4],
    order_id: u128,
}
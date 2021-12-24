const solanaWeb3 = require('@solana/web3.js');
const dotenv = require('dotenv');
const {
    TransactionInstruction, PublicKey, Transaction, Keypair, Connection, sendAndConfirmTransaction, SystemProgram,
    SYSVAR_RENT_PUBKEY
} = require("@solana/web3.js");
const fs = require("fs");
const {initializeLocalPayer, printProgramLogsForSignature, calculateMarketId} = require("./helpers");
const borsh = require("borsh");
dotenv.config({path: __dirname + '/.env'});
const md5 = require("md5")
const base58 = require("base58-js")
const programId = new PublicKey(process.env.PROGRAM_ID);
const {Token, AccountLayout} = require("@solana/spl-token");
const {InitializeTradeMarket, TradeMarketState, Trader} = require("./layouts");
const {binary_to_base58} = require("base58-js");
const BN = require("bn.js");
const serum = require("@project-serum/serum");
const {OpenOrders, DexInstructions, Market} = require("@project-serum/serum");
const markets = require("../../ninjaPrototcol/ninjadex-v3/serium-lib/markets.json");
(async function () {
    const connection = new Connection("https://api.mainnet-beta.solana.com");
    let serumProgramId = new PublicKey("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin")

    let results = []
    for(let m of markets) {
        let market = await serum.Market.load(connection, new PublicKey(m.address), {}, serumProgramId)
        let res = {
            name: m.name,
            address: m.address,
            eventQueue: market.decoded.eventQueue.toBase58(),
            requestQueue: market.decoded.requestQueue.toBase58(),
            bids: market.decoded.bids.toBase58(),
            asks: market.decoded.asks.toBase58(),
            baseMint: market.decoded.baseMint.toBase58(),
            quoteMint: market.decoded.quoteMint.toBase58(),
        }
        results.push(res)
    }

    console.log(JSON.stringify(results))


})()
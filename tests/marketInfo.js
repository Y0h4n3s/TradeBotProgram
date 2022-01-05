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
const {binary_to_base58} = require("base58-js");
const {TradeMarketState, CloseTradeMarket, Trader, Trade} = require("./layouts");
const {Market, OpenOrders} = require("@project-serum/serum");
const BN = require("bn.js");

(async function () {
    const connection = new Connection("https://api.mainnet-beta.solana.com");
   // let payer = await initializeLocalPayer(connection);
    let tokenProgramId = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
    let serumProgramId = new PublicKey("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin")
    let marketAddress = new PublicKey("J4oPt5Q3FYxrznkXLkbosAWrJ4rZLqJpGqz7vZUL4eMM")
    let signer = new Keypair()

    let market = await Market.load(connection, marketAddress, {}, serumProgramId)

    console.log(market.decoded.baseLotSize.toNumber(), market.decoded.quoteLotSize.toNumber())

})()
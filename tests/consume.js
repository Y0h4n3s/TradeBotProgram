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
    const connection = new Connection("https://api.devnet.solana.com");
    let payer = await initializeLocalPayer(connection);
    let tokenProgramId = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
    let serumProgramId = new PublicKey("73A1rYyFwTpRzEsGjJc1P45ee7qMo8vXuMZUDC42Wzwe")
    let marketAddress = new PublicKey("HuXUgd1E9bV1Dh9u1djgGcNybDK4q4Hp5nHtZ16VQdpa")
    let signer = new Keypair()

    let market = await Market.load(connection, marketAddress, undefined, serumProgramId)
    let openOrdersAccounts = (await market.findOpenOrdersAccountsForOwner(connection, payer.publicKey, 10000000000000)).map(op => op.address);
    let events = await market.loadEventQueue(connection);
    while (true) {
        for (let event of events) {
            let tx = new Transaction()

            let consumeIx = market.makeConsumeEventsInstruction([event.openOrders], 100)
            tx.add(consumeIx)
            console.log("Processing Transaction...")
            let sig = await sendAndConfirmTransaction(connection, tx, [payer]);
            await printProgramLogsForSignature(connection, sig)
            console.log("sig: ", sig, "\n")

        }

    }

})()
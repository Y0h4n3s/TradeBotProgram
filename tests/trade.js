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
    let tx = new Transaction()
    let tokenProgramId = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
    let serumProgramId = new PublicKey("73A1rYyFwTpRzEsGjJc1P45ee7qMo8vXuMZUDC42Wzwe")
    let marketAddress = new PublicKey("BYvVg2HW8gFT1kpEBbDqMTa7pfd2LJxHyFRvYHKWeg5E")
    let signer = new Keypair()

    let traders = await connection.getProgramAccounts(programId, {filters: [
            {dataSize: Trader.span},
            {
                memcmp: {
                    offset: 0,bytes: "BYvVg2HW8gFT1kpEBbDqMTa7pfd2LJxHyFRvYHKWeg5E"
                }
            },
            {
                memcmp: {
                    offset: Trader.offsetOf("owner"), bytes: payer.publicKey.toBase58()
                }
            },
            {
                memcmp: {
                    offset: Trader.offsetOf("status"), bytes: binary_to_base58(new Uint8Array([1])).toString()
                }
            }

        ]})
    let trader = traders[0]
    let decodedTrader = Trader.decode(trader.account.data, 0)


    let sermarket = await Market.load(connection, marketAddress, {}, serumProgramId)

    let myOrders = await sermarket.loadOrdersForOwner(connection, decodedTrader.marketSigner, 200000000000000000000);
    let decodedMarket = sermarket.decoded
    const data = {
    }
    console.log(myOrders)
    let tx_ix = Buffer.alloc(Trade.span)
    Trade.encode(data, tx_ix)
    console.log(tx_ix)
    let triggerTradeIx = new TransactionInstruction({
        data: tx_ix,
        keys: [
            {pubkey: decodedTrader.baseMarketWallet, isSigner: false, isWritable: true},
            {pubkey: decodedTrader.quoteMarketWallet, isSigner: false, isWritable: true},
            {pubkey: decodedTrader.marketSigner, isSigner: false, isWritable: true},
            {pubkey: marketAddress, isSigner: false, isWritable: true},
            {pubkey: decodedMarket.requestQueue, isSigner: false, isWritable: true},
            {pubkey: decodedMarket.eventQueue, isSigner: false, isWritable: true},
            {pubkey: decodedMarket.bids, isSigner: false, isWritable: true},
            {pubkey: decodedMarket.asks, isSigner: false, isWritable: true},
            {pubkey: decodedMarket.baseVault, isSigner: false, isWritable: true},
            {pubkey: decodedMarket.quoteVault, isSigner: false, isWritable: true},
            {pubkey: trader.pubkey, isSigner: false, isWritable: true},
            {pubkey: decodedTrader.serumOpenOrders, isSigner: false, isWritable: true},
            {pubkey: tokenProgramId, isSigner: false, isWritable: false},
            {pubkey: serumProgramId, isSigner: false, isWritable: false},
            {pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false}
        ],
        programId: programId

    })

    tx.add(triggerTradeIx)

    console.log("Processing Transaction...")
    let result = await sendAndConfirmTransaction(connection, tx, [payer])
    console.log("Result signature: ", result)
    await printProgramLogsForSignature(connection, result)


})()
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
const {TradeMarketState, CloseTradeMarket} = require("./layouts");
const {Market, OpenOrders} = require("@project-serum/serum");

(async function () {
    const connection = new Connection("https://api.devnet.solana.com");
    let payer = await initializeLocalPayer(connection);
    let tx = new Transaction()
    let tokenProgramId = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
    let serumProgramId = new PublicKey("DESVgJVGajEgKGXhb6XmqDHGz3VjdgP7rEVESBgxmroY")
    let marketAddress = new PublicKey("B8P4TGd4EW2TFpGNyEGmbRdRSQkLWKFjpYqfrjFFy5FQ")


    let serumMarkets = await connection.getProgramAccounts(serumProgramId, {filters: [
            {dataSize: Market.getLayout(serumProgramId).span},
            {memcmp: {
                offset: Market.getLayout(serumProgramId).offsetOf("ownAddress"),bytes: "7xF5H6mWt3mjPFshYPrdrx2vkUDXPLei3zdQYioBdvzX"
                }}
        ]})
        // serumMarkets.forEach(m => {
        //     let decoded = Market.getLayout(serumProgramId).decode(m.account.data)
        //     console.log(decoded)
        // })
    let market = serumMarkets[0]
    let decodedMarket = Market.getLayout(serumProgramId).decode(market.account.data)
    let sermarket = await Market.load(connection, decodedMarket.ownAddress, {}, serumProgramId)
    let savedMarkets = await connection.getProgramAccounts(programId, {
        filters: [{
            dataSize: TradeMarketState.span
        }, {
            memcmp: {
                offset: TradeMarketState.offsetOf("owner"), bytes: payer.publicKey.toBase58()
            },
        }, {
            memcmp: {
                offset: TradeMarketState.offsetOf("status"), bytes: binary_to_base58(new Uint8Array([1])).toString()
            }
        }]
    })
    let decoded = TradeMarketState.decode(savedMarkets[0].account.data)

    let bids = await sermarket.loadBids(connection);
    let asks = await sermarket.loadAsks(connection);

    console.log(Number(asks.getL2(1)[0][0]).toFixed(4), Number(bids.getL2(1)[0][0]).toFixed(4))

    let triggerTradeIx = new TransactionInstruction({
        data: Buffer.alloc(0),
        keys: [
            {pubkey: decoded.owner, isSigner: true, isWritable: true},
            {pubkey: decoded.marketState, isSigner: false, isWritable: true},
            {pubkey: decoded.baseMarketWallet, isSigner: false, isWritable: true},
            {pubkey: decoded.quoteMarketWallet, isSigner: false, isWritable: true},
            {pubkey: decoded.marketSigner, isSigner: false, isWritable: true},
            {pubkey: decoded.baseOwnerAccount, isSigner: false, isWritable: true},
            {pubkey: decoded.quoteOwnerAccount, isSigner: false, isWritable: true},
            {pubkey: decoded.address, isSigner: false, isWritable: true},
            {pubkey: decoded.openOrdersAccount, isSigner: false, isWritable: true},
            {pubkey: decoded.serumOpenOrdersAccount, isSigner: false, isWritable: true},
            {pubkey: decodedMarket.requestQueue, isSigner: false, isWritable: true},
            {pubkey: decodedMarket.eventQueue, isSigner: false, isWritable: true},
            {pubkey: decodedMarket.bids, isSigner: false, isWritable: true},
            {pubkey: decodedMarket.asks, isSigner: false, isWritable: true},
            {pubkey: decodedMarket.baseVault, isSigner: false, isWritable: true},
            {pubkey: decodedMarket.quoteVault, isSigner: false, isWritable: true},
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
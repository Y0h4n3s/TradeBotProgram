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
    let marketAddress = new PublicKey("7V5TrUp3wQp4hwNiQMMR4KEG98y4LTvDxXLNU64PnjbN")
    let signer = new Keypair()

    let traders = await connection.getProgramAccounts(programId, {
        filters: [
            {dataSize: Trader.span},
            {
                memcmp: {
                    offset: 0, bytes: "7V5TrUp3wQp4hwNiQMMR4KEG98y4LTvDxXLNU64PnjbN"
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

        ]
    })
    let trader = traders[0]
    let decodedTrader = Trader.decode(trader.account.data, 0)
    console.log("baseMarketWallet:", decodedTrader.baseMarketWallet.toBase58())
    console.log("quoteMarketWallet:", decodedTrader.quoteMarketWallet.toBase58())
    console.log("serumOpenOrders:", decodedTrader.serumOpenOrders.toBase58())
    console.log("traderSigner:", decodedTrader.marketSigner.toBase58())
    console.log("openOrderPairs:", decodedTrader.openOrderPairs.toNumber())
    console.log("baseBalance:", decodedTrader.baseBalance.toNumber())
    console.log("quoteBalance:", decodedTrader.quoteBalance.toNumber())
    console.log("totalTxs:", decodedTrader.totalTxs.toNumber())
    console.log("startingBase:", decodedTrader.startingBaseBalance.toNumber())
    console.log("startingQuote:", decodedTrader.startingQuoteBalance.toNumber())
    console.log(OpenOrders.getLayout(serumProgramId).span, OpenOrders.getLayout(serumProgramId).offsetOf("owner"))
    let market = await Market.load(connection, decodedTrader.marketAddress, undefined, serumProgramId)
    console.log("\nMarketBaseVault:", market.decoded.baseVault.toBase58())
    console.log("MarketQuoteVault:", market.decoded.quoteVault.toBase58())
    const bids = await market.loadBids(connection)
    const asks = await market.loadAsks(connection)// Retrieving fills
    const fills = await market.loadFills(connection);
    const serumOpenOrders = await OpenOrders.load(connection, decodedTrader.serumOpenOrders, serumProgramId)
    console.log("totalBaseBalanceLocked:", serumOpenOrders.baseTokenTotal.toNumber())
    console.log("totalQuoteBalanceLocked:", serumOpenOrders.quoteTokenTotal.toNumber())
    const openOrders =  market.filterForOpenOrders(bids, asks, [serumOpenOrders])
    for (let order of openOrders) {
        console.log(order.size, order.price, order.clientId.toNumber(), order.feeTier)
    }

    let events = await market.loadEventQueue(connection)
    console.log(events.length)
//     for (let fill of fills) {
//   console.log(fill.orderId, fill.price, fill.size, fill.side);
// }
    console.log(fills.length)
    //
    // // Retrieving fills
    // for (let fill of await market.loadFills(connection)) {
    //     console.log(fill.orderId, fill.price, fill.size, fill.side);
    // }
    // for (let [price, size] of bids.getL2(20)) {
    //     console.log(price, size);
    // }
    //
    // for (let order of asks) {
    //     console.log(
    //         order.orderId,
    //         order.price,
    //         order.size,
    //         order.side, // 'buy' or 'sell'
    //     );
    // }

})()
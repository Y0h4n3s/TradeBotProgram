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
const {TradeMarketState, CloseTradeMarket, Trader} = require("./layouts");
const {Market, OpenOrders} = require("@project-serum/serum");
const web3_js_1 = require("@solana/web3.js");
const buffer_1 = require("buffer");

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
    let marketState = await connection.getAccountInfo(decodedTrader.marketState);
    let decodedMarketState = TradeMarketState.decode(marketState.data, 0)

    const vaultSigner = await PublicKey.createProgramAddress([
        sermarket.address.toBuffer(),
        sermarket.decoded.vaultSignerNonce.toArrayLike(Buffer, 'le', 8),
    ], serumProgramId);

    let settleIx = new TransactionInstruction({
        data: Buffer.alloc(80),
        keys: [
            {pubkey: trader.pubkey, isSigner: false, isWritable: true},
            {pubkey: marketAddress, isSigner: false, isWritable: true},
            {pubkey: decodedTrader.serumOpenOrders, isSigner: false, isWritable: true},
            {pubkey: sermarket.decoded.bids, isSigner: false, isWritable: true},
            {pubkey: sermarket.decoded.asks, isSigner: false, isWritable: true},
            {pubkey: decodedTrader.marketSigner, isSigner: false, isWritable: true},
            {pubkey: sermarket.decoded.baseVault, isSigner: false, isWritable: true},
            {pubkey: sermarket.decoded.quoteVault, isSigner: false, isWritable: true},
            {pubkey: decodedTrader.baseMarketWallet, isSigner: false, isWritable: true},
            {pubkey: decodedTrader.quoteMarketWallet, isSigner: false, isWritable: true},
            {pubkey: vaultSigner, isSigner: false, isWritable: true},
            {pubkey: serumProgramId, isSigner: false, isWritable: false},
            {pubkey: tokenProgramId, isSigner: false, isWritable: false},
            {pubkey: marketAddress, isSigner: false, isWritable: false},
            {pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false}
        ],
        programId: programId

    })


    tx.add(settleIx)

    console.log("Processing Transaction...")
    sendAndConfirmTransaction(connection, tx, [payer]).then(async sig => {
        console.log("accounts created: ", sig)
        await printProgramLogsForSignature(connection, sig)

    }).catch(console.error)

})()
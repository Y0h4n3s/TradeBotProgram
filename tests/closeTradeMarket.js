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

(async function () {
    const connection = new Connection("https://api.devnet.solana.com");
    let payer = await initializeLocalPayer(connection);
    let tx = new Transaction()
    let tokenProgramId = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")

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
    console.log(savedMarkets.length)
    let decoded = TradeMarketState.decode(savedMarkets[0].account.data)
    let b= Buffer.alloc(32)
    CloseTradeMarket.encode({marketState: decoded.marketState}, b)
    let closeMarketIx = new TransactionInstruction({
        data: b,
        keys: [
            {pubkey: decoded.marketState, isSigner: false, isWritable: true}
        ],
        programId: programId

    })

    tx.add(closeMarketIx)

    console.log("Processing Transaction...")
    let result = await sendAndConfirmTransaction(connection, tx, [payer])
    console.log("Result signature: ", result)
    await printProgramLogsForSignature(connection, result)


})();

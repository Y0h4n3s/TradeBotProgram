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
const { TradeMarketState, Trader, InitializeTrader, UpdateTrader} = require("./layouts");
const {binary_to_base58} = require("base58-js");
const BN = require("bn.js");
const serum = require("@project-serum/serum");
const {OpenOrders, DexInstructions, Market} = require("@project-serum/serum");
(async function () {
    const connection = new Connection("https://api.devnet.solana.com");
    let payer = await initializeLocalPayer(connection);
    let tx = new Transaction()
    let tokenProgramId = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
    let serumProgramId = new PublicKey("73A1rYyFwTpRzEsGjJc1P45ee7qMo8vXuMZUDC42Wzwe")
    let systemProgramId = new PublicKey("11111111111111111111111111111111")
    let payerBaseTokenAccount = new PublicKey('CEzHF6839TYwH4KcQgS77GfyfJdkHVBDgBxgYg9SEH64')
    let payerQuoteTokenAccount = new PublicKey('FpGxUvLJtwu9XKkfYXNMJeUzJ9KgLmDmEEXk9JXVkEkG')
    let marketAddress = new PublicKey("BYvVg2HW8gFT1kpEBbDqMTa7pfd2LJxHyFRvYHKWeg5E")

    let traders = await connection.getProgramAccounts(programId, {
        filters: [
            {dataSize: Trader.span},
            {
                memcmp: {
                    offset: 0, bytes: "BYvVg2HW8gFT1kpEBbDqMTa7pfd2LJxHyFRvYHKWeg5E"
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
    let sermarket = await Market.load(connection, marketAddress, {}, serumProgramId)
    let data = {
        tradeProfit: new BN(2 * Math.pow(10, sermarket._quoteSplTokenDecimals)),
        stoppingPrice: sermarket.priceNumberToLots(4),
        simultaneousOpenPositions: new BN('120'),

    }

    let tx_data = Buffer.alloc(UpdateTrader.span)
    UpdateTrader.encode(data, tx_data)

    let tx_ix = new TransactionInstruction({
        data: tx_data,
        keys: [{
            pubkey: payer.publicKey,
            isSigner: true,
            isWritable: true
        }, {
            pubkey: trader.pubkey,
            isSigner: false,
            isWritable: true
        }],
        programId: programId
    })

    tx.add(tx_ix)

    tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash


    console.log("Processing Transaction...")
    sendAndConfirmTransaction(connection, tx, [payer]).then(async sig => {
        console.log("accounts created: ", sig)
        await printProgramLogsForSignature(connection, sig)

    }).catch(console.error)

})()
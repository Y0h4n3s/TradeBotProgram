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
(async function () {
    const connection = new Connection("https://api.devnet.solana.com");
    let payer = await initializeLocalPayer(connection);
    let tx = new Transaction()
    let tokenProgramId = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
    let serumProgramId = new PublicKey("73A1rYyFwTpRzEsGjJc1P45ee7qMo8vXuMZUDC42Wzwe")


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
                    offset: Trader.offsetOf("status"), bytes: binary_to_base58(new Uint8Array([1])).toString()
                }
            }

        ]
    })
    console.log(traders.length)

    let trader = traders.at(0)

    let decommissionIx = new TransactionInstruction({
        programId: programId,
        data: Buffer.alloc(56),
        keys: [
            {
                pubkey: payer.publicKey,
                isSigner: false,
                isWritable: true
            }, {
                pubkey: trader.pubkey,
                isSigner: false,
                isWritable: true
            }
        ]


    })

    tx.add(decommissionIx)

    tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash


    console.log("Processing Transaction...")
    sendAndConfirmTransaction(connection, tx, [payer]).then(async sig => {
        console.log("accounts created: ", sig)
        await printProgramLogsForSignature(connection, sig)

    }).catch(err=> {
        console.error(err)
    })
})()
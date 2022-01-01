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
const {InitializeTradeMarket, TradeMarketState, RegisterTrader, Trader} = require("./layouts");
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



    let serumMarkets = await connection.getProgramAccounts(programId, {filters: [
            {dataSize: 129},
            {memcmp: {
                    offset: 0,bytes: "HuXUgd1E9bV1Dh9u1djgGcNybDK4q4Hp5nHtZ16VQdpa"
                }}
        ]})

    let market = serumMarkets[0]
    let decodedMarket = TradeMarketState.decode(market.account.data, 0);
    const data = {
        registerDate: new BN(new Date().getTime() / 1000)
    }
    let tx_data = Buffer.alloc(8)
    RegisterTrader.encode(data, tx_data)
    let traderWallet = new Keypair()

    let createTraderAccountIx = SystemProgram.createAccount({
        space: Trader.span,
        lamports: await connection.getMinimumBalanceForRentExemption(
            Trader.span
        ),
        fromPubkey: payer.publicKey,
        newAccountPubkey: traderWallet.publicKey,
        programId: programId
    })

    let tx_ix = new TransactionInstruction({
        data: tx_data,
        keys: [{
            pubkey: payer.publicKey,
            isSigner: true,
            isWritable: true
        }, {
            pubkey: market.pubkey,
            isSigner: false,
            isWritable: true
        }, {
            pubkey: traderWallet.publicKey,
            isSigner: false,
            isWritable: true
        },
        ],
        programId
    })
    tx.add(createTraderAccountIx)

    tx.add(tx_ix)

    tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash


    console.log("Processing Transaction...")
    sendAndConfirmTransaction(connection, tx, [payer, traderWallet]).then(async sig => {
        console.log("accounts created: ", sig)
        await printProgramLogsForSignature(connection, sig)

    })

})()
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
const {InitializeTradeMarket, TradeMarketState} = require("./layouts");
const {binary_to_base58} = require("base58-js");
const BN = require("bn.js");
const serum = require("@project-serum/serum");
const {OpenOrders, DexInstructions} = require("@project-serum/serum");
(async function () {
    const connection = new Connection("https://api.devnet.solana.com");
    let payer = await initializeLocalPayer(connection);
    let tx = new Transaction()
    let tokenProgramId = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
    let serumProgramId = new PublicKey("73A1rYyFwTpRzEsGjJc1P45ee7qMo8vXuMZUDC42Wzwe")






    let market = new PublicKey("HuXUgd1E9bV1Dh9u1djgGcNybDK4q4Hp5nHtZ16VQdpa")

    let tx_data = Buffer.alloc(0)

    let baseToken = new PublicKey("EdnAnrrvnS42MZNrj4wpDqE6XAJMg3sADPEcqgyAzJ9H")
    let quoteToken = new PublicKey("J4TYCGMWfEUJCg3PmYn6KTjksai6cv45JP3AitYxpRY5")

    //let [marketAccount, seed] = await PublicKey.findProgramAddress([my_seed], programId)
    //console.log(my_seed, seed)
    let marketAccount = new Keypair()
    let createMarketAccountIx = SystemProgram.createAccount({
        space: TradeMarketState.span,
        lamports: await connection.getMinimumBalanceForRentExemption(
            TradeMarketState.span
        ),
        fromPubkey: payer.publicKey,
        newAccountPubkey: marketAccount.publicKey,
        programId: programId
    })

    let tx_ix = new TransactionInstruction({
        data: tx_data,
        keys: [{
            pubkey: payer.publicKey,
            isSigner: true,
            isWritable: true
        }, {
            pubkey: marketAccount.publicKey,
            isSigner: false,
            isWritable: true
        }, {
            pubkey: baseToken,
            isSigner: false,
            isWritable: true
        }, {
            pubkey: quoteToken,
            isSigner: false,
            isWritable: true
        },
            {
             pubkey: market,
             isSigner: false,
             isWritable: true
            }],
        programId
    })

    tx.add(createMarketAccountIx)
    tx.add(tx_ix)

    tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash


    console.log("Processing Transaction...")
    sendAndConfirmTransaction(connection, tx, [payer, marketAccount]).then(async sig => {
        console.log("accounts created: ", sig)
        await printProgramLogsForSignature(connection, sig)

    })




})()





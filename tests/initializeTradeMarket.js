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
    let serumProgramId = new PublicKey("DESVgJVGajEgKGXhb6XmqDHGz3VjdgP7rEVESBgxmroY")






    let market = new PublicKey("B8P4TGd4EW2TFpGNyEGmbRdRSQkLWKFjpYqfrjFFy5FQ")

    const data = {
        tradeProfit: new BN('0'),
        stoppingPrice: new BN('1999'),
        startingPriceBuy: new BN('10000'),
        startingPriceSell: new BN('200000'),
        simultaneousOpenPositions: new BN('200000'),
        startingBaseBalance: new BN("100"),
        startingQuoteBalance: new BN("200"),
        startingValue: new BN("300"),
        status: new BN('2'),
        address: market,
    }
    let tx_data = Buffer.alloc(97)
    InitializeTradeMarket.encode(data, tx_data)

    let payerBaseTokenAccount = new PublicKey('CEzHF6839TYwH4KcQgS77GfyfJdkHVBDgBxgYg9SEH64')
    let payerQuoteTokenAccount = new PublicKey('FpGxUvLJtwu9XKkfYXNMJeUzJ9KgLmDmEEXk9JXVkEkG')
    let baseToken = new PublicKey("EdnAnrrvnS42MZNrj4wpDqE6XAJMg3sADPEcqgyAzJ9H")
    let quoteToken = new PublicKey("J4TYCGMWfEUJCg3PmYn6KTjksai6cv45JP3AitYxpRY5")

    //let [marketAccount, seed] = await PublicKey.findProgramAddress([my_seed], programId)
    //console.log(my_seed, seed)
    let marketAccount = new Keypair()
    let baseMarketWallet = new Keypair()
    let quoteMarketWallet = new Keypair()
    let openOrdersAccount = new Keypair()
    let serumOpenOrdersAccount = new Keypair()
    let createMarketAccountIx = SystemProgram.createAccount({
        space: TradeMarketState.span,
        lamports: await connection.getMinimumBalanceForRentExemption(
            TradeMarketState.span
        ),
        fromPubkey: payer.publicKey,
        newAccountPubkey: marketAccount.publicKey,
        programId: programId
    })

    let createOpenOrdersAccountIx = SystemProgram.createAccount({
        space: 1104,
        lamports: await connection.getMinimumBalanceForRentExemption(
            1104
        ),
        fromPubkey: payer.publicKey,
        newAccountPubkey: openOrdersAccount.publicKey,
        programId: programId
    })
    let createSerumOpenOrdersAccountIx = SystemProgram.createAccount({
        space: 3228,
        lamports: await connection.getMinimumBalanceForRentExemption(
            3228
        ),
        fromPubkey: payer.publicKey,
        newAccountPubkey: serumOpenOrdersAccount.publicKey,
        programId: serumProgramId
    })

    let createBaseMarketWalletAccountIx = SystemProgram.createAccount({
        space: AccountLayout.span,
        lamports: await connection.getMinimumBalanceForRentExemption(
            AccountLayout.span
        ),
        fromPubkey: payer.publicKey,
        newAccountPubkey: baseMarketWallet.publicKey,
        programId: tokenProgramId
    })

    let createQuoteMarketWalletAccountIx = SystemProgram.createAccount({
        space: AccountLayout.span,
        lamports: await connection.getMinimumBalanceForRentExemption(
            AccountLayout.span
        ),
        fromPubkey: payer.publicKey,
        newAccountPubkey: quoteMarketWallet.publicKey,
        programId: tokenProgramId
    })

    const initBaseMarketWalletAccountIx = Token.createInitAccountInstruction(
        tokenProgramId,
        baseToken,
        baseMarketWallet.publicKey,
        payer.publicKey
    );

    const initQuoteMarketWalletAccountIx = Token.createInitAccountInstruction(
        tokenProgramId,
        quoteToken,
        quoteMarketWallet.publicKey,
        payer.publicKey
    );

    const transferBaseTokensToBaseMarketWalletIx = Token.createTransferInstruction(
        tokenProgramId,
        payerBaseTokenAccount,
        baseMarketWallet.publicKey,
        payer.publicKey,
        [],
        100000
     );
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
        }, {
            pubkey: baseMarketWallet.publicKey,
            isSigner: false,
            isWritable: true
        }, {
            pubkey: quoteMarketWallet.publicKey,
            isSigner: false,
            isWritable: true
        }, {
            pubkey: payerBaseTokenAccount,
            isSigner: false,
            isWritable: true
        }, {
            pubkey: payerQuoteTokenAccount,
            isSigner: false,
            isWritable: true
        },{
            pubkey: openOrdersAccount.publicKey,
            isSigner: false,
            isWritable: true
        },{
            pubkey: serumOpenOrdersAccount.publicKey,
            isSigner: false,
            isWritable: true
        }, {
            pubkey: market,
            isSigner: false,
            isWritable: false
        },{
            pubkey: tokenProgramId,
            isSigner: false,
            isWritable: false
        },
        ],
        programId
    })
    tx.add(createMarketAccountIx)
    tx.add(createOpenOrdersAccountIx)
    tx.add(createSerumOpenOrdersAccountIx)
    tx.add(createBaseMarketWalletAccountIx)
    tx.add(createQuoteMarketWalletAccountIx)


    tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash


    console.log("Processing Transaction...")
     sendAndConfirmTransaction(connection, tx, [payer, marketAccount, openOrdersAccount, serumOpenOrdersAccount, baseMarketWallet, quoteMarketWallet]).then(async sig => {
         console.log("accounts created: ", sig)
         await printProgramLogsForSignature(connection, sig)

         let tx2 = new Transaction()
         tx2.add(initBaseMarketWalletAccountIx)
         tx2.add(initQuoteMarketWalletAccountIx)
         tx2.add(transferBaseTokensToBaseMarketWalletIx)
         tx2.add(tx_ix)

         let result = await sendAndConfirmTransaction(connection, tx2, [payer]);
         console.log("Result Signature: ", result)

         await printProgramLogsForSignature(connection, result)

     })





})()





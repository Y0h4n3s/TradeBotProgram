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
const { TradeMarketState, Trader, InitializeTrader} = require("./layouts");
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
    let marketAddress = new PublicKey("HuXUgd1E9bV1Dh9u1djgGcNybDK4q4Hp5nHtZ16VQdpa")

    let traders = await connection.getProgramAccounts(programId, {filters: [
            {dataSize: Trader.span},
        {
                memcmp: {
                    offset: 0,bytes: "HuXUgd1E9bV1Dh9u1djgGcNybDK4q4Hp5nHtZ16VQdpa"
                }
            },
            {
                memcmp: {
                    offset: Trader.offsetOf("owner"), bytes: payer.publicKey.toBase58()
                }
            },

            {
                memcmp: {
                    offset: Trader.offsetOf("status"), bytes: binary_to_base58(new Uint8Array([0])).toString()
                }
            }

        ]})

    let trader = traders[0]
    let decodedTrader = Trader.decode(trader.account.data, 0)

    let serumMarkets = await connection.getProgramAccounts(programId, {filters: [
            {dataSize: 129},
            {memcmp: {
                    offset: 0,bytes: "HuXUgd1E9bV1Dh9u1djgGcNybDK4q4Hp5nHtZ16VQdpa"
                }}
        ]})

    let market = serumMarkets[0]
    let decodedMarket = TradeMarketState.decode(market.account.data, 0);
    let sermarket = await Market.load(connection, marketAddress, {}, serumProgramId)

    let minSerumOpenOrdersAccountRent = await connection.getMinimumBalanceForRentExemption(
        3228
    )
    console.log(new BN(6000 * Math.pow(10, sermarket._baseSplTokenDecimals)))
    const data = {
        tradeProfit: new BN(1 * Math.pow(10, sermarket._quoteSplTokenDecimals)),
        stoppingPrice: sermarket.priceNumberToLots(5),
        startingPriceBuy: sermarket.priceNumberToLots(6.0),
        startingPriceSell:sermarket.priceNumberToLots(6.5),
        simultaneousOpenPositions: new BN('120'),
        startingBaseBalance: new BN(6000 * Math.pow(10, sermarket._baseSplTokenDecimals)),
        startingQuoteBalance:new BN(36000 * Math.pow(10, sermarket._quoteSplTokenDecimals)),
        startingValue: new BN("30000000"),
        serumOpenOrdersRent: new BN(`${minSerumOpenOrdersAccountRent}`)
    }
    console.log(data)
    let tx_data = Buffer.alloc(InitializeTrader.span)
    InitializeTrader.encode(data, tx_data)


    let baseToken = new PublicKey("EdnAnrrvnS42MZNrj4wpDqE6XAJMg3sADPEcqgyAzJ9H")
    let quoteToken = new PublicKey("J4TYCGMWfEUJCg3PmYn6KTjksai6cv45JP3AitYxpRY5")

    let baseMarketWallet = new Keypair()
    let quoteMarketWallet = new Keypair()
    let serumOpenOrdersAccount = new Keypair()



    console.log(serumOpenOrdersAccount.publicKey.toBase58(), decodedTrader.marketSigner.toBase58(), minSerumOpenOrdersAccountRent)

    let transferToPdaForRentIx = SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: decodedTrader.marketSigner,
        lamports: minSerumOpenOrdersAccountRent
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
        },{
            pubkey: baseMarketWallet.publicKey,
            isSigner: false,
            isWritable: true
        }, {
            pubkey: quoteMarketWallet.publicKey,
            isSigner: false,
            isWritable: true
        }, {
            pubkey: trader.pubkey,
            isSigner: false,
            isWritable: true
        }, {
            pubkey: decodedTrader.marketSigner,
            isSigner: false,
            isWritable: true
        }, {
            pubkey: serumOpenOrdersAccount.publicKey,
            isSigner: true,
            isWritable: true
        }, {
            pubkey: payerBaseTokenAccount,
            isSigner: false,
            isWritable: true
        }, {
            pubkey: payerQuoteTokenAccount,
            isSigner: false,
            isWritable: true
        }, {
            pubkey: tokenProgramId,
            isSigner: false,
            isWritable: false
        },{
            pubkey: serumProgramId,
            isSigner: false,
            isWritable: false
        },{
            pubkey: programId,
            isSigner: false,
            isWritable: false
        },{
            pubkey: systemProgramId,
            isSigner: false,
            isWritable: false
        },
        ],
        programId
    })
    tx.add(transferToPdaForRentIx)
    tx.add(createBaseMarketWalletAccountIx)
    tx.add(createQuoteMarketWalletAccountIx)
    tx.add(initBaseMarketWalletAccountIx)
    tx.add(initQuoteMarketWalletAccountIx)
    tx.add(tx_ix)

    tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash


    console.log("Processing Transaction...")
    sendAndConfirmTransaction(connection, tx, [payer, serumOpenOrdersAccount, baseMarketWallet, quoteMarketWallet]).then(async sig => {
        console.log("accounts created: ", sig)
        await printProgramLogsForSignature(connection, sig)

    }).catch(console.error)

})()
const dotenv = require('dotenv');
const {initializeLocalPayer, printProgramLogsForSignature, calculateMarketId, initializeLocalPayerAccount} = require("./helpers");
dotenv.config({path: __dirname + '/.env'});
const serum = require("@project-serum/serum");
const {OpenOrders} = require("@project-serum/serum");
const Token = require("@solana/spl-token").Token;
const TOKEN_PROGRAM_ID = require("@solana/spl-token").TOKEN_PROGRAM_ID;
const TokenInstructions = require("@project-serum/serum").TokenInstructions;
const Market = require("@project-serum/serum").Market;
const DexInstructions = require("@project-serum/serum").DexInstructions;
const web3 = require("@project-serum/anchor").web3;
const Connection = web3.Connection;
const BN = require("@project-serum/anchor").BN;
const serumCmn = require("@project-serum/common");
const {sendAndConfirmTransaction} = require("@solana/web3.js");
const Account = web3.Account;
const Transaction = web3.Transaction;
const PublicKey = web3.PublicKey;
const SystemProgram = web3.SystemProgram;
const DEX_PID = new PublicKey("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin");

async function getVaultOwnerAndNonce(marketPublicKey, dexProgramId = DEX_PID) {
    const nonce = new BN(0);
    while (nonce.toNumber() < 255) {
        try {
            const vaultOwner = await PublicKey.createProgramAddress(
                [marketPublicKey.toBuffer(), nonce.toArrayLike(Buffer, "le", 8)],
                dexProgramId
            );
            return [vaultOwner, nonce];
        } catch (e) {
            nonce.iaddn(1);
        }
    }
    throw new Error("Unable to find nonce");
}
async function signTransactions({
                                    transactionsAndSigners,
                                    wallet,
                                    connection,
                                }) {
    const blockhash = (await connection.getRecentBlockhash("max")).blockhash;
    transactionsAndSigners.forEach(({ transaction, signers = [] }) => {
        transaction.recentBlockhash = blockhash;
        transaction.setSigners(
            wallet.publicKey,
            ...signers.map((s) => s.publicKey)
        );
        if (signers.length > 0) {
            transaction.partialSign(...signers);
        }
    });
    return await wallet.signAllTransactions(
        transactionsAndSigners.map(({ transaction }) => transaction)
    );
}

(async function () {
    const connection = new Connection("https://api.devnet.solana.com");
    let payer = await initializeLocalPayerAccount(connection);
    let tx = new Transaction()
    let tokenProgramId = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
    let serumProgramId = new PublicKey("73A1rYyFwTpRzEsGjJc1P45ee7qMo8vXuMZUDC42Wzwe")

    let baseLotSize = 100000000
    let quoteLotSize = 100
    let feeRateBps = 0
    let baseMint = new PublicKey("EdnAnrrvnS42MZNrj4wpDqE6XAJMg3sADPEcqgyAzJ9H")
    let quoteMint = new PublicKey("FepzwZGPorzLXnvCQAhnHWC53UBz7SCQcvw7FQmbPJRg")
    let payerBaseTokenAccount = new PublicKey('CEzHF6839TYwH4KcQgS77GfyfJdkHVBDgBxgYg9SEH64')
    let payerQuoteTokenAccount = new PublicKey('FpGxUvLJtwu9XKkfYXNMJeUzJ9KgLmDmEEXk9JXVkEkG')
    const market = new Account();
    const requestQueue = new Account();
    const eventQueue = new Account();
    const bids = new Account();
    const asks = new Account();
    const baseVault = new Account();
    const quoteVault = new Account();
    const quoteDustThreshold = new BN(100);

    const [vaultOwner, vaultSignerNonce] = await getVaultOwnerAndNonce(
        market.publicKey,
        serumProgramId
    );

    const tx1 = new Transaction();
    tx1.add(
        SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: baseVault.publicKey,
            lamports: await connection.getMinimumBalanceForRentExemption(165),
            space: 165,
            programId: TOKEN_PROGRAM_ID,
        }),
        SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: quoteVault.publicKey,
            lamports: await connection.getMinimumBalanceForRentExemption(165),
            space: 165,
            programId: TOKEN_PROGRAM_ID,
        }),
        TokenInstructions.initializeAccount({
            account: baseVault.publicKey,
            mint: baseMint,
            owner: vaultOwner,
        }),
        TokenInstructions.initializeAccount({
            account: quoteVault.publicKey,
            mint: quoteMint,
            owner: vaultOwner,
        })
    );

    let initMarketIxOld = DexInstructions.initializeMarket({
        market: market.publicKey,
        requestQueue: requestQueue.publicKey,
        eventQueue: eventQueue.publicKey,
        bids: bids.publicKey,
        asks: asks.publicKey,
        baseVault: baseVault.publicKey,
        quoteVault: quoteVault.publicKey,
        baseMint,
        quoteMint,
        baseLotSize: new BN(baseLotSize),
        quoteLotSize: new BN(quoteLotSize),
        feeRateBps,
        vaultSignerNonce,
        quoteDustThreshold,
        programId: serumProgramId,
    })

    const tx2 = new Transaction();
    tx2.add(
        SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: market.publicKey,
            lamports: await connection.getMinimumBalanceForRentExemption(
                Market.getLayout(serumProgramId).span
            ),
            space: Market.getLayout(serumProgramId).span,
            programId: serumProgramId,
        }),
        SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: requestQueue.publicKey,
            lamports: await connection.getMinimumBalanceForRentExemption(5120 + 12),
            space: 5120 + 12,
            programId: serumProgramId,
        }),
        SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: eventQueue.publicKey,
            lamports: await connection.getMinimumBalanceForRentExemption(262144 + 12),
            space: 262144 + 12,
            programId: serumProgramId,
        }),
        SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: bids.publicKey,
            lamports: await connection.getMinimumBalanceForRentExemption(65536 + 12),
            space: 65536 + 12,
            programId: serumProgramId,
        }),
        SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: asks.publicKey,
            lamports: await connection.getMinimumBalanceForRentExemption(65536 + 12),
            space: 65536 + 12,
            programId: serumProgramId,
        }),
    );
    let initializeMarketIx = new web3.TransactionInstruction({
        data: initMarketIxOld.data,
        keys: initMarketIxOld.keys,
        programId:serumProgramId
    })
    tx2.add(initializeMarketIx)
    console.log("Processing Transaction...")
    sendAndConfirmTransaction(connection, tx1, [payer, baseVault, quoteVault]).then(async sig => {
        console.log("accounts created: ", sig)
        await printProgramLogsForSignature(connection, sig)

        let result = await sendAndConfirmTransaction(connection, tx2, [payer, market, requestQueue, eventQueue, asks,bids]);
        console.log("Result Signature: ", result)

        await printProgramLogsForSignature(connection, result)
        const acc = await connection.getAccountInfo(market.publicKey);
        console.log(acc, market.publicKey.toBase58())

    })

//HuXUgd1E9bV1Dh9u1djgGcNybDK4q4Hp5nHtZ16VQdpa

})()
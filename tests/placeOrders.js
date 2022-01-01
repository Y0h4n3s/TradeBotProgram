const {Market} = require("@project-serum/serum");
const {sendAndConfirmTransaction, PublicKey, Connection} = require("@solana/web3.js");
const {initializeLocalPayerAccount, printProgramLogsForSignature} = require("./helpers");

(async () => {
    const connection = new Connection("https://api.devnet.solana.com");
    let payer = await initializeLocalPayerAccount(connection);
    let serumProgramId = new PublicKey("73A1rYyFwTpRzEsGjJc1P45ee7qMo8vXuMZUDC42Wzwe")
    let market = new PublicKey("HuXUgd1E9bV1Dh9u1djgGcNybDK4q4Hp5nHtZ16VQdpa")
    let sermarket = await Market.load(connection, market, {}, serumProgramId)
    let payerBaseTokenAccount = new PublicKey('CEzHF6839TYwH4KcQgS77GfyfJdkHVBDgBxgYg9SEH64')
    let payerQuoteTokenAccount = new PublicKey('FpGxUvLJtwu9XKkfYXNMJeUzJ9KgLmDmEEXk9JXVkEkG')
    let asks = [
        [3.041, 7.8],
        [3.051, 72.3],
        [3.055, 5.4],
        [3.067, 15.7],
        [3.077, 390.0],
        [3.09, 24.0],
        [3.11, 36.3],
        [3.133, 300.0],
        [3.167, 687.8],
    ];
    let bids = [
        [2.4, 18500],
        [2.5, 11290],
        [2.7, 16200],
        [2.8, 11503],
        [2.9, 18208],
        [2.961, 12054],
    ];

    for (let k = 0; k < asks.length; k += 1) {
        let ask = asks[k];
        const sig = await sermarket.placeOrder(connection, {
            owner: payer,
            payer: payerBaseTokenAccount,
            side: "sell",
            price: ask[0],
            size: ask[1],
            orderType: "postOnly",
            clientId: undefined,
            openOrdersAddressKey: undefined,
            openOrdersAccount: undefined,
            feeDiscountPubkey: null,
            selfTradeBehavior: "abortTransaction",
        });
        console.log(sig)
    }

    for (let k = 0; k < bids.length; k += 1) {
        let bid = bids[k];
        const sig = await sermarket.placeOrder(connection, {
            owner: payer,
            payer: payerQuoteTokenAccount,
            side: "buy",
            price: bid[0],
            size: bid[1],
            orderType: "postOnly",
            clientId: undefined,
            openOrdersAddressKey: undefined,
            openOrdersAccount: undefined,
            feeDiscountPubkey: null,
            selfTradeBehavior: "abortTransaction",
        });
        console.log(sig)

    }
})()

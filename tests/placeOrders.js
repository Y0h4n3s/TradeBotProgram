const {Market} = require("@project-serum/serum");
const {sendAndConfirmTransaction, PublicKey, Connection} = require("@solana/web3.js");
const {initializeLocalPayerAccount, printProgramLogsForSignature} = require("./helpers");

(async () => {
    const connection = new Connection("https://api.devnet.solana.com");
    let payer = await initializeLocalPayerAccount(connection);
    let serumProgramId = new PublicKey("DESVgJVGajEgKGXhb6XmqDHGz3VjdgP7rEVESBgxmroY")
    let market = new PublicKey("7xF5H6mWt3mjPFshYPrdrx2vkUDXPLei3zdQYioBdvzX")
    let sermarket = await Market.load(connection, market, {}, serumProgramId)
    let payerBaseTokenAccount = new PublicKey('CEzHF6839TYwH4KcQgS77GfyfJdkHVBDgBxgYg9SEH64')
    let payerQuoteTokenAccount = new PublicKey('FpGxUvLJtwu9XKkfYXNMJeUzJ9KgLmDmEEXk9JXVkEkG')
    let asks = [
        [6.041, 7.8],
        [6.051, 72.3],
        [6.055, 5.4],
        [6.067, 15.7],
        [6.077, 390.0],
        [6.09, 24.0],
        [6.11, 36.3],
        [6.133, 300.0],
        [6.167, 687.8],
    ];
    let bids = [
        [6.004, 8.5],
        [5.995, 12.9],
        [5.987, 6.2],
        [5.978, 15.3],
        [5.965, 82.8],
        [5.961, 25.4],
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

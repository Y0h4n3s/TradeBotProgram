



function uptrend(start, end, increment) {
    let trades = []
    for(let i = start; i < end; i+= increment) {
        trades.push([i, i+increment])
    }
    return trades
}
function downtrend(start, end, increment) {
    let trades = []
    for(let i = start; i > end; i-= increment) {
        trades.push([i, i+increment])
    }
    return trades
}

let trader = {
    min_trade_profit: 10000000, simultaneous_open_positions: 120, base_balance: 20000000, quote_balance: 120000000000, open_order_pairs: 0
}
let trader1 = {
    min_trade_profit: 10000000, simultaneous_open_positions: 120, base_balance: 20000000, quote_balance: 120000000000, open_order_pairs: 0
}

let bullish = [...uptrend(6000, 7000, 20)]
let bearish = [...uptrend(6000, 6400, 20), ...downtrend(6400, 5000, 20)]
let sideways = [...uptrend(6000, 6400, 20), ...downtrend(6400, 5900, 20),...uptrend(5900, 6600, 20), ...downtrend(6600, 6200, 20)]
function simulateForTrades(trades) {

    let buy_queue = []
    let sell_queue = []
    for( let [idx, [buy_price, sell_price]] of Object.entries(trades)) {
        if (trader.open_order_pairs*2 >= trader.simultaneous_open_positions) {
            for (let i = 0; i < (Math.min(20, buy_queue.length)); i ++) {
                trader.base_balance += sell_queue.shift()
                trader.quote_balance += buy_queue.shift()
            }
            trader.open_order_pairs -= 20;
        }
        let market_price = (buy_price + sell_price) / 2;
        let base_size = trader.base_balance / (trader.simultaneous_open_positions - trader.open_order_pairs * 2);
        let quote_size =trader.quote_balance /(trader.simultaneous_open_positions - trader.open_order_pairs * 2);
        let price_gap_buy = market_price - ((quote_size * market_price) / (trader.min_trade_profit / 2 + quote_size));
        let price_gap_sell = ((trader.min_trade_profit + 2 * base_size * market_price) / (2 * base_size)) - market_price;

        let order_buy_price = market_price - price_gap_buy;
        let order_sell_price = market_price + price_gap_sell;
        // console.log(`Placing orders @ buy ${order_buy_price}, sell ${order_sell_price} ${price_gap_sell}`)
        // console.log(`Sold base_size ${base_size} for ${base_size * order_sell_price}`)
        // console.log(`Bought ${quote_size / order_buy_price} for ${quote_size}`)
        // console.log(quote_size, base_size)
        // console.log("")
        trader.base_balance -= base_size;
        sell_queue.push((quote_size / order_buy_price));
        trader.quote_balance -= quote_size;
        buy_queue.push(base_size * order_sell_price);
        trader.open_order_pairs += 2;
    }
    buy_queue.forEach(e => trader.quote_balance += e)
    sell_queue.forEach(e => trader.base_balance += e)
    console.log("")
    console.log("Before", trader1)
    console.log("After", trader)
    let buySum = 0
    let sellSum = 0
    for( let [buy_price, sell_price] of trades) {
        buySum += buy_price;
        sellSum += sell_price;
    }

    let averagePrice = ((buySum / trades.length) + (sellSum / trades.length)) / 2
    let price = (trades.at(-1)[0] + trades.at(-1)[1])/2
    console.log("AveragePrice:", averagePrice)
    console.log("CurrentPrice:", price)
    let startingAverageQuote = trader1.quote_balance + trader1.base_balance * averagePrice;
    let finalAverageQuote = trader.quote_balance + trader.base_balance * averagePrice;
    let startingQuote = trader1.quote_balance + trader1.base_balance * price;
    let finalQuote = trader.quote_balance + trader.base_balance * price;
    console.log(`Starting average quote balance: ${startingAverageQuote}`)
    console.log(`Final average quote balance: ${startingAverageQuote}`)
    console.log(`diff: ${finalAverageQuote - startingAverageQuote}`)
    console.log(`diff usd: ${(finalAverageQuote - startingAverageQuote) / Math.pow(10, 6)}`)

 console.log(`\nStarting quote balance: ${startingQuote}`)
    console.log(`Final quote balance: ${finalQuote}`)
    console.log(`diff: ${finalQuote - startingQuote}`)
    console.log(`diff usd: ${(finalQuote - startingQuote) / Math.pow(10, 6)}`)
    console.log("\n\n")

}


function simulatePrices(marketPrice, tradeProfit, quoteLotSize, baseLotSize, baseSizeLots) {
    profitLots = (tradeProfit / marketPrice) / quoteLotSize
    let priceGap =  (baseLotSize / quoteLotSize) * Math.exp((((-1/baseSizeLots)/marketPrice)*tradeProfit*(Math.log(baseLotSize))))
    const buyPrice = marketPrice - priceGap
    const sellPrice = marketPrice + priceGap
    console.log(priceGap, buyPrice, sellPrice, profitLots)
}
simulatePrices(185000, 50000, 100, 100000000, 678571428)

const bufferLayout = require("buffer-layout");
const {PublicKey} = require("@solana/web3.js");
const BN = require("bn.js");
const buffer = require("buffer");



class PublicKeyLayout extends bufferLayout.Layout {

    constructor(property) {
        super(32, property)
    }

    getSpan(b, offset) {
        let span = this.span;
        if (0 > span) {
            span = this.length.decode(b, offset);
        }
        return span;
    }

    decode(b, offset) {
        return new PublicKey(b.slice(offset, offset + this.span))
    }

    encode(src, b, offset) {

        if (undefined === offset) {
            offset = 0;
        }

        if (undefined === buffer) {
            b = Buffer.alloc(this.span);
        }
        if ((offset + this.span) > b.length) {
            throw new RangeError('encoding overruns Buffer');
        }
        //return b.write(src.toBytes(), offset, "binary")
        let keyBuffer = src.toBuffer()
        for (let i = 0; i < keyBuffer.length; i++) {
            b[i + offset] = keyBuffer[i]
        }

        return this.span
    }
}

class U64 extends bufferLayout.Layout {

    constructor(property, span=8) {
        super(span, property)
    }

    getSpan(b, offset) {
        let span = this.span;
        if (0 > span) {
            span = this.length.decode(b, offset);
        }
        return span;
    }

    decode(b, offset) {

        return new BN(b.slice(offset, offset + this.span), "le")
    }


    encode(src, b, offset) {

        if (undefined === offset) {
            offset = 0;
        }

        if (undefined === buffer) {
            b = Buffer.alloc(this.span);
        }
        if ((offset + this.span) > b.length) {
            throw new RangeError('encoding overruns Buffer');
        }
        let bu = src.toArrayLike(Buffer, "le", 8)
        for (let i = 0; i < bu.length; i++) {
            b[i + offset] = bu[i]
        }

        return this.span
    }
}

class U128 extends U64 {
    constructor(property) {
        super(property, 16)
    }
}

const TradeMarketState = bufferLayout.struct([
    new PublicKeyLayout( "address"),
    new PublicKeyLayout("baseMint"),
    new PublicKeyLayout("quoteMint"),
    new PublicKeyLayout("owner"),
    bufferLayout.u8("status")
])

const CloseTradeMarket = bufferLayout.struct([
    new PublicKeyLayout("marketState")
])

const RegisterTrader = bufferLayout.struct([
    new U64("registerDate")
])

const InitializeTrader = bufferLayout.struct([
    new U64("tradeProfit"),
    new U64("stoppingPrice"),
    new U64("startingPriceBuy"),
    new U64("startingPriceSell"),
    new U64("simultaneousOpenPositions"),
    new U64("startingBaseBalance"),
    new U64("startingQuoteBalance"),
    new U64("startingValue"),
    new U64("serumOpenOrdersRent"),
])

const Trade = bufferLayout.struct([
    bufferLayout.blob(128, "_padding")


])

const UpdateTrader = bufferLayout.struct([
    new U64("tradeProfit"),
    new U64("stoppingPrice"),
    new U64("simultaneousOpenPositions"),
    bufferLayout.blob(65, "_padding")

])
const Trader = bufferLayout.struct([
    new PublicKeyLayout("marketAddress"),
    new PublicKeyLayout("baseMarketWallet"),
    new PublicKeyLayout("quoteMarketWallet"),
    new PublicKeyLayout("serumOpenOrders"),
    new PublicKeyLayout("marketSigner"),
    new PublicKeyLayout("marketState"),
    new PublicKeyLayout("owner"),
    new U64("tradeProfit"),
    new U64("stoppingPrice"),
    new U64("startingPriceBuy"),
    new U64("startingPriceSell"),
    new U64("simultaneousOpenPositions"),
    new U64("startingBaseBalance"),
    new U64("startingQuoteBalance"),
    new U64("startingValue"),
    new U64("baseBalance"),
    new U64("quoteBalance"),
    new U64("value"),
    new U64("openOrderPairs"),
    new U64("totalTxs"),
    new U64("registerDate"),
    bufferLayout.u8("status"),
    bufferLayout.blob(128, "_padding")
])

module.exports = {TradeMarketState, CloseTradeMarket, InitializeTrader, RegisterTrader, Trader, Trade, UpdateTrader}
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

    constructor(property) {
        super(8, property)
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
        return b.write(src.toString("hex"), offset, "hex")
    }
}

const TradeMarketState = bufferLayout.struct([
    new PublicKeyLayout( "address"),
    new PublicKeyLayout("baseMint"),
    new PublicKeyLayout("baseMarketWallet"),
    new PublicKeyLayout("quoteMint"),
    new PublicKeyLayout("quoteMarketWallet"),
    new PublicKeyLayout("baseOwnerAccount"),
    new PublicKeyLayout("quoteOwnerAccount"),
    new PublicKeyLayout("marketSigner"),
    new PublicKeyLayout("marketState"),
    new PublicKeyLayout("owner"),
    new PublicKeyLayout("openOrdersAccount"),
    new PublicKeyLayout("serumOpenOrdersAccount"),
    new U64("tradeProfit"),
    new U64("stoppingPrice"),
    new U64("startingPriceBuy"),
    new U64("startingPriceSell"),
    new U64("simultaneousOpenPositions"),
    new U64("startingBaseBalance"),
    new U64("startingQuoteBalance"),
    new U64("startingValue"),
    bufferLayout.u8("status")
])

const CloseTradeMarket = bufferLayout.struct([
    new PublicKeyLayout("marketState")
])

const InitializeTradeMarket = bufferLayout.struct([
    new PublicKeyLayout("address"),
    new U64("tradeProfit"),
    new U64("stoppingPrice"),
    new U64("startingPriceBuy"),
    new U64("startingPriceSell"),
    new U64("simultaneousOpenPositions"),
    bufferLayout.u8("status"),
    new U64("startingBaseBalance"),
    new U64("startingQuoteBalance"),
    new U64("startingValue"),
])

const Trade = bufferLayout.struct([
    new U64("sellPrice"),
    new U64("buyPrice")
])

module.exports = {TradeMarketState, CloseTradeMarket, InitializeTradeMarket}
const fs = require("fs");
const {Keypair, Account} = require("@solana/web3.js");
const {base58_to_binary, binary_to_base58} = require("base58-js");

async function createKeypairFromFile(
    filePath,
) {
    const secretKeyString = await fs.readFileSync(filePath);
    const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
    return Keypair.fromSecretKey(secretKey);
}

async function createAccountFromFile(filePath) {
    const secretKeyString = await fs.readFileSync(filePath);
    const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
    return new Account(secretKey)
}

async function printProgramLogsForSignature(connection, signature) {
    let output = await connection.getTransaction(signature)
    console.log("************  Program Logs  ************")
    console.log(output['meta']['logMessages'].reduce((acc, log) => acc += "\n" + log))
}

async function initializeLocalPayer(connection) {
    const keypair = await createKeypairFromFile('/home/y0h4n3s/.config/solana/id.json')
    let balance = await connection.getBalance(keypair.publicKey);
    if (balance < 1000000000) {
        console.log("Requesting 1 Sol airdrop for payer: ", keypair.publicKey.toBase58())
        await connection.requestAirdrop(keypair.publicKey, 1000000000);
        console.log("Airdrop Successful")
    }
    return keypair
}

async function initializeLocalPayerAccount(connection) {
    const keypair = await createAccountFromFile('/home/y0h4n3s/.config/solana/id.json')
    let balance = await connection.getBalance(keypair.publicKey);
    if (balance < 1000000000) {
        console.log("Requesting 1 Sol airdrop for payer: ", keypair.publicKey.toBase58())
        await connection.requestAirdrop(keypair.publicKey, 1000000000);
        console.log("Airdrop Successful")
    }
    return keypair
}

function calculateMarketId(market, owner) {
    let marketBuffer = Buffer.from(market.toBuffer())
    let ownerBuffer = Buffer.from(owner.toBuffer())
    let idBuffer = Buffer.alloc(32)
    for (let i = 0; i < marketBuffer.length; i++) {
        idBuffer[i] = marketBuffer[i] + ownerBuffer[i]
    }

    return idBuffer

}

module.exports = {initializeLocalPayer, printProgramLogsForSignature, createKeypairFromFile, calculateMarketId, initializeLocalPayerAccount}
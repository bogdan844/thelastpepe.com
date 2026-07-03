# The Last Pepe - Smart Contract Deployment Guide

## Overview
This guide covers deploying the PEPE presale contracts on both Ethereum and Solana blockchains.

---

## 📋 Prerequisites

### For Ethereum (Solidity)
- Node.js (v16+)
- Hardhat or Truffle
- OpenZeppelin Contracts
- MetaMask or similar wallet

### For Solana (Rust)
- Rust (latest)
- Anchor Framework
- Solana CLI
- Phantom or Solflare wallet

---

## 🔗 ETHEREUM DEPLOYMENT

### Step 1: Set Up Hardhat Project

```bash
npm init -y
npm install --save-dev hardhat @nomicfoundation/hardhat-toolbox
npm install @openzeppelin/contracts ethers
npx hardhat
```

### Step 2: Configure hardhat.config.js

```javascript
require("@nomicfoundation/hardhat-toolbox");
require("dotenv").config();

module.exports = {
  solidity: "0.8.20",
  networks: {
    mainnet: {
      url: process.env.MAINNET_RPC_URL,
      accounts: [process.env.PRIVATE_KEY],
    },
    sepolia: {
      url: process.env.SEPOLIA_RPC_URL,
      accounts: [process.env.PRIVATE_KEY],
    },
  },
};
```

### Step 3: Create .env File

```env
PRIVATE_KEY=your_wallet_private_key_here
MAINNET_RPC_URL=https://eth.public-rpc.com
SEPOLIA_RPC_URL=https://sepolia.infura.io/v3/YOUR_INFURA_KEY
ETHERSCAN_API_KEY=your_etherscan_api_key
```

### Step 4: Deploy PEPE Token First

Create `contracts/PepeToken.sol`:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract PepeToken is ERC20, Ownable {
    constructor(uint256 initialSupply) ERC20("The Last Pepe", "PEPE") {
        _mint(msg.sender, initialSupply * 10 ** decimals());
    }

    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }
}
```

### Step 5: Create Deployment Script

Create `scripts/deploy.js`:

```javascript
const hre = require("hardhat");

async function main() {
  console.log("Deploying PEPE Token...");
  
  const PepeToken = await hre.ethers.getContractFactory("PepeToken");
  const pepeToken = await PepeToken.deploy(hre.ethers.utils.parseEther("1000000000")); // 1B tokens
  await pepeToken.deployed();
  console.log("PEPE Token deployed to:", pepeToken.address);

  console.log("Deploying Presale Contract...");
  
  const PepePresale = await hre.ethers.getContractFactory("PepePresale");
  const presale = await PepePresale.deploy(
    pepeToken.address,
    "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    "0xYourTreasuryWalletAddress"
  );
  await presale.deployed();
  console.log("Presale deployed to:", presale.address);

  // Transfer presale tokens to presale contract
  await pepeToken.transfer(presale.address, hre.ethers.utils.parseEther("500000000"));
  console.log("Transferred PEPE tokens to presale contract");

  // Save addresses
  const addresses = {
    pepeToken: pepeToken.address,
    presale: presale.address,
  };

  const fs = require("fs");
  fs.writeFileSync("deployment.json", JSON.stringify(addresses, null, 2));
  console.log("Deployment addresses saved to deployment.json");
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
```

### Step 6: Deploy

```bash
npx hardhat run scripts/deploy.js --network sepolia
```

### Step 7: Verify on Etherscan

```bash
npx hardhat verify --network sepolia PRESALE_CONTRACT_ADDRESS \
  PEPE_TOKEN_ADDRESS \
  USDT_ADDRESS \
  USDC_ADDRESS \
  TREASURY_ADDRESS
```

---

## 🌊 SOLANA DEPLOYMENT

### Step 1: Set Up Anchor Project

```bash
anchor init pepe-presale
cd pepe-presale
```

### Step 2: Update Cargo.toml

```toml
[package]
name = "pepe_presale"
version = "0.1.0"
edition = "2021"

[dependencies]
anchor-lang = "0.29"
anchor-spl = "0.29"

[lib]
crate-type = ["cdylib", "rlib"]
```

### Step 3: Set Network in Anchor.toml

```toml
[programs.devnet]
pepe_presale = "11111111111111111111111111111111"

[programs.mainnet]
pepe_presale = "YOUR_PROGRAM_ID_HERE"

[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha --timeout 1000000 tests/**/*.ts"
```

### Step 4: Build Program

```bash
anchor build
```

### Step 5: Get Program ID

```bash
solana address -k target/deploy/pepe_presale-keypair.json
```

Update this in `Anchor.toml` and `lib.rs` declare_id!

### Step 6: Deploy to Devnet

```bash
anchor deploy --provider.cluster devnet
```

### Step 7: Deploy to Mainnet

```bash
anchor deploy --provider.cluster mainnet
```

---

## 🔧 POST-DEPLOYMENT SETUP

### On Ethereum:

1. **Approve USDT/USDC Spending:**
   ```javascript
   const usdtABI = ["function approve(address spender, uint256 amount) public returns (bool)"];
   const usdt = new ethers.Contract(USDT_ADDRESS, usdtABI, signer);
   await usdt.approve(PRESALE_ADDRESS, ethers.constants.MaxUint256);
   ```

2. **Set Configuration:**
   ```javascript
   const presale = await ethers.getContractAt("PepePresale", PRESALE_ADDRESS);
   await presale.setBonusPercentage(30); // 30% bonus
   await presale.setPresalePrice(ethers.utils.parseEther("0.001")); // $0.001 per token
   ```

### On Solana:

1. **Initialize Presale:**
   ```javascript
   const tx = await program.methods
     .initializePresale(
       new BN(1000000), // price in lamports
       30, // bonus percentage
       new BN(500000000000000) // cap
     )
     .accounts({
       presaleConfig: presaleConfigPDA,
       authority: provider.wallet.publicKey,
     })
     .rpc();
   ```

2. **Create Token Vault:**
   ```bash
   spl-token create-account <PEPE_MINT_ADDRESS>
   ```

---

## 📝 Configuration Checklist

- [ ] Deploy PEPE token on Ethereum
- [ ] Deploy Presale on Ethereum
- [ ] Deploy Presale on Solana
- [ ] Update `index.html` with contract addresses
- [ ] Update `index.html` with correct RPC URLs
- [ ] Set presale price
- [ ] Set bonus percentage
- [ ] Set presale cap
- [ ] Transfer tokens to presale contracts
- [ ] Test transactions on testnet
- [ ] Verify contracts on Etherscan
- [ ] Whitelist initial users (if required)

---

## 🧪 Testing

### Ethereum - Hardhat Test

Create `test/PepePresale.test.js`:

```javascript
const { expect } = require("chai");

describe("PepePresale", function () {
  it("Should allow buying with ETH", async function () {
    const [owner, buyer] = await ethers.getSigners();
    
    // Deploy contracts
    const PepeToken = await ethers.getContractFactory("PepeToken");
    const pepeToken = await PepeToken.deploy(ethers.utils.parseEther("1000000"));
    
    const PepePresale = await ethers.getContractFactory("PepePresale");
    const presale = await PepePresale.deploy(
      pepeToken.address,
      "0xdAC17F958D2ee523a2206206994597C13D831ec7",
      "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      owner.address
    );
    
    // Transfer tokens to presale
    await pepeToken.transfer(presale.address, ethers.utils.parseEther("500000"));
    
    // Buy with ETH
    const tx = await presale.connect(buyer).buyWithETH({
      value: ethers.utils.parseEther("1")
    });
    
    expect(tx).to.emit(presale, "TokensPurchased");
  });
});
```

Run tests:
```bash
npx hardhat test
```

### Solana - Anchor Test

Tests are in `tests/pepe-presale.ts`

```bash
anchor test
```

---

## 🚀 GOING LIVE

### Before Mainnet:

1. **Audit:** Consider getting a professional audit
2. **Test Thoroughly:** Use testnet extensively
3. **Security:** Enable pause functions and whitelist
4. **Documentation:** Create user guides
5. **Support:** Set up Discord/Telegram support

### Mainnet Launch:

1. **Deploy contracts**
2. **Update website with contract addresses**
3. **Announce on social media**
4. **Monitor transactions**
5. **Be ready for support queries**

---

## 📞 Support & Resources

- **Ethereum**: https://ethereum.org/
- **Solana**: https://solana.com/
- **Hardhat Docs**: https://hardhat.org/
- **Anchor Docs**: https://www.anchor-lang.com/
- **OpenZeppelin**: https://docs.openzeppelin.com/

---

## ⚠️ Security Notes

1. **Never commit private keys** - use `.env` files
2. **Test on testnet first** - always
3. **Use multisig wallets** for treasury
4. **Enable pause functions** for emergencies
5. **Implement rate limiting** if needed
6. **Keep contracts upgraded** - use proxy pattern if needed

---

Last Updated: 2024
The Last Pepe Team

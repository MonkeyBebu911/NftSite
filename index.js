const express = require('express');
const bodyParser = require('body-parser');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { ContractPromise } = require('@polkadot/api-contract');
const fs = require('fs');

const app = express();
const port = 3000;

// Middleware
app.use(bodyParser.json());

// Initialize API and contract
let api;
let contract;

async function initializeApi() {
  const wsProvider = new WsProvider('wss://rococo-contracts-rpc.polkadot.io'); // Adjust this to your node's WebSocket endpoint
  api = await ApiPromise.create({ provider: wsProvider });
  
  // Replace with your contract's address and ABI
  const contractAddress = '5HL5sE2hGq8bxvkDXL8JQTcZBxt4tKTqgXMCS2yFwGQgKQ75';
  const abi = JSON.parse(fs.readFileSync('../nft_contract.json', 'utf8'));
  contract = new ContractPromise(api, abi, contractAddress);
}

initializeApi().catch(console.error);

// Route for getting an NFT
app.get('/nft', async (req, res) => {
  const { username, token_id } = req.query;
 
  if (!username || !token_id) {
    return res.status(400).json({ error: 'Username and token_id are required' });
  }

  try {
    const gasLimit = api.registry.createType('WeightV2', {
      refTime: 1000000000n,
      proofSize: 50000n,
    });

    const { result, output } = await contract.query.getNft(null, {
      gasLimit,
      storageDepositLimit: null,
    }, token_id);

    if (result.isOk && output) {
      const nft = output.toJSON();
      console.log(nft);  // Keep this for debugging

      if (!nft || !nft.ok) {
        return res.status(404).json({ error: 'NFT not found' });
      }

      const nftData = nft.ok;  // The actual NFT data is nested under 'ok'

      if (nftData.username.toLowerCase() === username.toLowerCase()) {
        return res.json({ success: true, item: nftData.item });
      } else {
        return res.status(403).json({ error: 'Username does not match the NFT owner' });
      }
    } else {
      return res.status(404).json({ error: 'NFT not found' });
    }
  } catch (error) {
    console.error('Error querying contract:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Start the server
app.listen(port, () => {
  console.log(`Server running at http://localhost:${port}`);
});
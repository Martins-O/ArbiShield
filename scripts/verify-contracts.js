const axios = require("axios");
const fs = require("fs");
const path = require("path");

const CONTRACTS = [
  {
    name: "DetectionEngine",
    address: "0x700f33B8fECc0eC61cFAA44F8e6c9BdF751C4e9E",
    filename: "DetectionEngine.sol"
  },
  {
    name: "CircuitBreaker", 
    address: "0x91b3611115ABc980BD1a4F8E16ad4EBDe61aB07d",
    filename: "CircuitBreaker.sol"
  },
  {
    name: "AlertRegistry",
    address: "0x02c5084D7fC06cE22746716df892Cde3E05Bb8E1",
    filename: "AlertRegistry.sol"
  }
];

const BLOCKSCOUT_API = "https://sepolia-explorer.arbitrum.io/api";

async function verifyContract(name, address, filename) {
  const contractPath = path.join(__dirname, "..", "contracts", filename);
  const sourceCode = fs.readFileSync(contractPath, "utf8");
  
  const params = {
    module: "contract",
    action: "verifysourcecode",
    contractaddress: address,
    sourceCode: sourceCode,
    codeformat: "solidity-single-file",
    compilerversion: "v0.8.20+commit.a1b79de6",
    optimizationUsed: "1",
    runs: "200",
    constructorArguements: "",
    contractname: name
  };

  try {
    console.log(`Verifying ${name}...`);
    const response = await axios.post(`${BLOCKSCOUT_API}?`, null, { params });
    
    console.log("  Response:", response.data);
    
    if (response.data.status === "1") {
      console.log(`  ✓ ${name} verification submitted! GUID: ${response.data.result}`);
      return response.data.result;
    } else {
      console.log(`  ✗ ${name} failed: ${response.data.result}`);
      return null;
    }
  } catch (error) {
    console.log(`  ✗ ${name} error:`, error.response?.data || error.message);
    return null;
  }
}

async function main() {
  console.log("=== Verifying Contracts via Blockscout API ===\n");
  
  for (const contract of CONTRACTS) {
    await verifyContract(contract.name, contract.address, contract.filename);
    await new Promise(r => setTimeout(r, 2000));
  }
}

main().catch(console.error);
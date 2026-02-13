const { ethers } = require("hardhat");

async function main() {
  const ALERT_REGISTRY = "0x02c5084D7fC06cE22746716df892Cde3E05Bb8E1";
  const alertRegistry = await ethers.getContractAt("AlertRegistry", ALERT_REGISTRY);

  console.log("Registering alert...");
  const tx = await alertRegistry.registerAlert(
    "ArbiShield Deployed",
    "Security contracts deployed to Arbitrum Sepolia testnet.",
    0
  );
  await tx.wait();
  console.log("Alert registered!");

  const stats = await alertRegistry.getSystemStats();
  console.log("Total alerts:", stats[0]);
}

main().catch(console.error);
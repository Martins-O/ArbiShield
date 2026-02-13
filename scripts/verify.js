const { ethers } = require("hardhat");

async function main() {
  const DETECTION_ENGINE = "0x700f33B8fECc0eC61cFAA44F8e6c9BdF751C4e9E";
  const CIRCUIT_BREAKER = "0x91b3611115ABc980BD1a4F8E16ad4EBDe61aB07d";
  const ALERT_REGISTRY = "0x02c5084D7fC06cE22746716df892Cde3E05Bb8E1";

  const detectionEngine = await ethers.getContractAt("DetectionEngine", DETECTION_ENGINE);
  const circuitBreaker = await ethers.getContractAt("CircuitBreaker", CIRCUIT_BREAKER);
  const alertRegistry = await ethers.getContractAt("AlertRegistry", ALERT_REGISTRY);

  console.log("=== ArbiShield Deployment Verification ===\n");
  console.log("Contracts deployed on Arbitrum Sepolia:\n");
  console.log("DetectionEngine:", DETECTION_ENGINE);
  console.log("CircuitBreaker:", CIRCUIT_BREAKER);
  console.log("AlertRegistry:", ALERT_REGISTRY);

  console.log("\n--- DetectionEngine ---");
  try {
    const owner = await detectionEngine.owner();
    console.log("Owner:", owner);
    const stats = await detectionEngine.getSystemStats();
    console.log("Active Metrics:", stats[0]);
    console.log("Total Anomalies:", stats[1]);
  } catch (e) {
    console.log("Error:", e.message);
  }

  console.log("\n--- CircuitBreaker ---");
  try {
    const owner = await circuitBreaker.owner();
    console.log("Owner:", owner);
    const status = await circuitBreaker.getStatus();
    console.log("Is Tripped:", status[0]);
    console.log("Active Conditions:", status[1]);
  } catch (e) {
    console.log("Error:", e.message);
  }

  console.log("\n--- AlertRegistry ---");
  try {
    const owner = await alertRegistry.owner();
    console.log("Owner:", owner);
    const stats = await alertRegistry.getSystemStats();
    console.log("Total Alerts:", stats[0]);
    console.log("Open Alerts:", stats[1]);
    console.log("Resolved Alerts:", stats[2]);
  } catch (e) {
    console.log("Error:", e.message);
  }

  console.log("\n✓ Deployment verified!");
  console.log("\nView on Arbiscan Sepolia:");
  console.log("https://sepolia.arbiscan.io/address/" + DETECTION_ENGINE);
}

main().catch(console.error);
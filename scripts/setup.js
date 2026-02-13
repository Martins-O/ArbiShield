const { ethers } = require("hardhat");

async function main() {
  const [deployer] = await ethers.getSigners();
  console.log("Setting up initial configuration...");
  console.log("Deployer:", deployer.address);

  const DETECTION_ENGINE = "0x700f33B8fECc0eC61cFAA44F8e6c9BdF751C4e9E";
  const CIRCUIT_BREAKER = "0x91b3611115ABc980BD1a4F8E16ad4EBDe61aB07d";
  const ALERT_REGISTRY = "0x02c5084D7fC06cE22746716df892Cde3E05Bb8E1";

  const detectionEngine = await ethers.getContractAt("DetectionEngine", DETECTION_ENGINE);
  const circuitBreaker = await ethers.getContractAt("CircuitBreaker", CIRCUIT_BREAKER);
  const alertRegistry = await ethers.getContractAt("AlertRegistry", ALERT_REGISTRY);

  console.log("\n1. Registering sample metric in DetectionEngine...");
  const metricTx = await detectionEngine.registerMetric(ethers.parseEther("1000"));
  await metricTx.wait();
  console.log("   Metric registered!");

  console.log("\n2. Adding condition to CircuitBreaker...");
  const conditionTx = await circuitBreaker.addCondition(5, 3600);
  await conditionTx.wait();
  console.log("   Condition added!");

  console.log("\n3. Registering sample alert in AlertRegistry...");
  const alertTx = await alertRegistry.registerAlert(
    "ArbiShield Deployed",
    "ArbiShield security contracts have been successfully deployed to Arbitrum Sepolia.",
    0
  );
  await alertTx.wait();
  console.log("   Alert registered!");

  console.log("\n=== Verification ===");
  console.log("DetectionEngine owner:", await detectionEngine.owner());
  console.log("CircuitBreaker owner:", await circuitBreaker.owner());
  console.log("AlertRegistry owner:", await alertRegistry.owner());

  const stats = await detectionEngine.getSystemStats();
  console.log("\nDetectionEngine stats - Active:", stats[0], "Anomalies:", stats[1]);

  const alertStats = await alertRegistry.getSystemStats();
  console.log("AlertRegistry stats - Total:", alertStats[0], "Open:", alertStats[1]);

  console.log("\n✓ All contracts verified and configured!");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
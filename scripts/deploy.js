const { ethers } = require("hardhat");

async function main() {
  console.log("Deploying ArbiShield contracts...");
  
  const [deployer] = await ethers.getSigners();
  console.log("Deploying contracts with the account:", deployer.address);
  
  // Get contract factories
  const DetectionEngine = await ethers.getContractFactory("DetectionEngine");
  const CircuitBreaker = await ethers.getContractFactory("CircuitBreaker");
  const AlertRegistry = await ethers.getContractFactory("AlertRegistry");
  
  // Deploy DetectionEngine
  console.log("Deploying DetectionEngine...");
  const detectionEngine = await DetectionEngine.deploy();
  await detectionEngine.waitForDeployment();
  const detectionEngineAddress = await detectionEngine.getAddress();
  console.log("DetectionEngine deployed to:", detectionEngineAddress);
  
  // Deploy CircuitBreaker
  console.log("Deploying CircuitBreaker...");
  const circuitBreaker = await CircuitBreaker.deploy();
  await circuitBreaker.waitForDeployment();
  const circuitBreakerAddress = await circuitBreaker.getAddress();
  console.log("CircuitBreaker deployed to:", circuitBreakerAddress);
  
  // Deploy AlertRegistry
  console.log("Deploying AlertRegistry...");
  const alertRegistry = await AlertRegistry.deploy();
  await alertRegistry.waitForDeployment();
  const alertRegistryAddress = await alertRegistry.getAddress();
  console.log("AlertRegistry deployed to:", alertRegistryAddress);
  
  // Initial setup
  console.log("Setting up initial configuration...");
  
  // Register a sample metric in DetectionEngine
  const metricTx = await detectionEngine.registerMetric(ethers.parseEther("1000"));
  await metricTx.wait();
  console.log("Sample metric registered in DetectionEngine");
  
  // Add a condition to CircuitBreaker
  const conditionTx = await circuitBreaker.addCondition(5, 3600); // 5 events in 1 hour
  await conditionTx.wait();
  console.log("Sample condition added to CircuitBreaker");
  
  // Register a sample alert in AlertRegistry
  const alertTx = await alertRegistry.registerAlert(
    "Initial Security Alert",
    "This is a test alert to verify the AlertRegistry is working correctly.",
    0 // LOW severity
  );
  await alertTx.wait();
  console.log("Sample alert registered in AlertRegistry");
  
  // Save deployment information
  const deploymentInfo = {
    network: await ethers.provider.getNetwork(),
    deployer: deployer.address,
    contracts: {
      DetectionEngine: detectionEngineAddress,
      CircuitBreaker: circuitBreakerAddress,
      AlertRegistry: alertRegistryAddress
    },
    timestamp: new Date().toISOString()
  };
  
  const fs = require("fs");
  fs.writeFileSync(
    "deployment-info.json", 
    JSON.stringify(deploymentInfo, null, 2)
  );
  
  console.log("\\n=== Deployment Summary ===");
  console.log("DetectionEngine:", detectionEngineAddress);
  console.log("CircuitBreaker:", circuitBreakerAddress);
  console.log("AlertRegistry:", alertRegistryAddress);
  console.log("Deployment info saved to deployment-info.json");
  console.log("\\nAll contracts deployed successfully!");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
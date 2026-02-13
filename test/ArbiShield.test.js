const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("ArbiShield Contracts", function () {
  let detectionEngine, circuitBreaker, alertRegistry;
  let owner, addr1, addr2;

  beforeEach(async function () {
    [owner, addr1, addr2] = await ethers.getSigners();

    // Deploy contracts
    const DetectionEngine = await ethers.getContractFactory("DetectionEngine");
    detectionEngine = await DetectionEngine.deploy();
    await detectionEngine.waitForDeployment();

    const CircuitBreaker = await ethers.getContractFactory("CircuitBreaker");
    circuitBreaker = await CircuitBreaker.deploy();
    await circuitBreaker.waitForDeployment();

    const AlertRegistry = await ethers.getContractFactory("AlertRegistry");
    alertRegistry = await AlertRegistry.deploy();
    await alertRegistry.waitForDeployment();
  });

  describe("DetectionEngine", function () {
    it("Should register a metric", async function () {
      const threshold = ethers.parseEther("1000");
      await expect(detectionEngine.registerMetric(threshold))
        .to.emit(detectionEngine, "MetricRegistered")
        .withArgs(0, threshold);
    });

    it("Should detect anomaly when value exceeds threshold", async function () {
      const threshold = ethers.parseEther("100");
      const metricId = await detectionEngine.registerMetric(threshold);
      
      const exceedingValue = ethers.parseEther("150");
      await expect(detectionEngine.updateMetric(metricId.value, exceedingValue))
        .to.emit(detectionEngine, "AnomalyDetected");
    });

    it("Should revert for invalid metric ID", async function () {
      await expect(detectionEngine.updateMetric(999, ethers.parseEther("100")))
        .to.be.revertedWithCustomError(detectionEngine, "InvalidMetricId");
    });
  });

  describe("CircuitBreaker", function () {
    it("Should add a trigger condition", async function () {
      const threshold = 5;
      const windowDuration = 3600;
      
      await expect(circuitBreaker.addCondition(threshold, windowDuration))
        .to.emit(circuitBreaker, "ConditionAdded")
        .withArgs(0, threshold, windowDuration);
    });

    it("Should trip circuit breaker when threshold is exceeded", async function () {
      const threshold = 2;
      const windowDuration = 3600;
      
      await circuitBreaker.addCondition(threshold, windowDuration);
      
      // Trigger multiple times
      await circuitBreaker.checkAndTrip(0, 100);
      await expect(circuitBreaker.checkAndTrip(0, 100))
        .to.emit(circuitBreaker, "CircuitTripped");
    });

    it("Should allow manual trip and reset", async function () {
      await expect(circuitBreaker.tripManual("Test trip"))
        .to.emit(circuitBreaker, "CircuitTripped");
      
      await expect(circuitBreaker.reset("Test reset"))
        .to.emit(circuitBreaker, "CircuitReset");
    });
  });

  describe("AlertRegistry", function () {
    it("Should register an alert", async function () {
      const title = "Security Alert";
      const description = "This is a test alert";
      const severity = 1; // MEDIUM
      
      const tx = await alertRegistry.registerAlert(title, description, severity);
      await expect(tx).to.emit(alertRegistry, "AlertRegistered");
    });

    it("Should resolve an alert", async function () {
      const tx = await alertRegistry.registerAlert("Test", "Test", 0);
      const receipt = await tx.wait();
      const alertId = receipt.logs[0].args[0];
      
      await expect(alertRegistry.resolveAlert(alertId, "Resolved for testing"))
        .to.emit(alertRegistry, "AlertResolved");
    });

    it("Should register unique alerts", async function () {
      await alertRegistry.registerAlert("Test Alert", "Test Description", 0);
      const countBefore = await alertRegistry.alertCount();
      
      await alertRegistry.registerAlert("Different Alert", "Different Description", 0);
      const countAfter = await alertRegistry.alertCount();
      
      expect(countAfter).to.equal(countBefore + 1n);
    });
  });

  describe("Integration", function () {
    it("Should coordinate between contracts", async function () {
      // Register metric
      const threshold = ethers.parseEther("100");
      const tx = await detectionEngine.registerMetric(threshold);
      const receipt = await tx.wait();
      const metricId = receipt.logs[0].args[0];
      
      // Trigger anomaly
      const exceedingValue = ethers.parseEther("150");
      await detectionEngine.updateMetric(metricId, exceedingValue);
      
      // Register alert about anomaly
      await alertRegistry.registerAlert(
        "Anomaly Detected",
        `Metric ${metricId} exceeded threshold`,
        2 // HIGH severity
      );
      
      // Verify system state
      const stats = await detectionEngine.getSystemStats();
      expect(stats[1]).to.equal(1n);
      
      const alertStats = await alertRegistry.getSystemStats();
      expect(alertStats[0]).to.equal(1n);
    });
  });
});
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title DetectionEngine
 * @notice Monitors metrics and detects anomalies for ArbiShield security system
 * @dev Tracks multiple metrics with configurable thresholds and anomaly detection
 */
contract DetectionEngine is Ownable, ReentrancyGuard {
    
    // Structs
    struct Metric {
        uint256 threshold;
        uint256 currentValue;
        uint256 lastUpdated;
        bool active;
    }
    
    struct Anomaly {
        uint256 metricId;
        uint256 detectedValue;
        uint256 threshold;
        uint256 timestamp;
        bool resolved;
    }
    
    // State variables
    mapping(uint256 => Metric) public metrics;
    mapping(uint256 => Anomaly[]) public metricAnomalies;
    
    uint256 public metricCount;
    uint256 public totalAnomalies;
    
    // Events
    event MetricRegistered(uint256 indexed metricId, uint256 threshold);
    event MetricUpdated(uint256 indexed metricId, uint256 oldValue, uint256 newValue);
    event AnomalyDetected(uint256 indexed metricId, uint256 value, uint256 threshold, uint256 timestamp);
    event AnomalyResolved(uint256 indexed metricId, uint256 anomalyIndex, uint256 timestamp);
    
    // Errors
    error InvalidMetricId();
    error MetricNotActive();
    error ThresholdZero();
    error NoAnomalies();
    
    constructor() {
        _transferOwnership(msg.sender);
        metricCount = 0;
        totalAnomalies = 0;
    }
    
    /**
     * @notice Register a new metric with detection threshold
     * @param threshold The value threshold for anomaly detection
     * @return metricId The ID of the registered metric
     */
    function registerMetric(uint256 threshold) external onlyOwner returns (uint256) {
        if (threshold == 0) revert ThresholdZero();
        
        uint256 metricId = metricCount;
        metrics[metricId] = Metric({
            threshold: threshold,
            currentValue: 0,
            lastUpdated: block.timestamp,
            active: true
        });
        
        metricCount++;
        emit MetricRegistered(metricId, threshold);
        return metricId;
    }
    
    /**
     * @notice Update metric value and check for anomalies
     * @param metricId The ID of the metric to update
     * @param newValue The new value for the metric
     */
    function updateMetric(uint256 metricId, uint256 newValue) external nonReentrant {
        if (metricId >= metricCount) revert InvalidMetricId();
        if (!metrics[metricId].active) revert MetricNotActive();
        
        Metric storage metric = metrics[metricId];
        uint256 oldValue = metric.currentValue;
        
        metric.currentValue = newValue;
        metric.lastUpdated = block.timestamp;
        
        // Check for anomaly
        if (newValue > metric.threshold) {
            Anomaly memory anomaly = Anomaly({
                metricId: metricId,
                detectedValue: newValue,
                threshold: metric.threshold,
                timestamp: block.timestamp,
                resolved: false
            });
            
            metricAnomalies[metricId].push(anomaly);
            totalAnomalies++;
            
            emit AnomalyDetected(metricId, newValue, metric.threshold, block.timestamp);
        }
        
        emit MetricUpdated(metricId, oldValue, newValue);
    }
    
    /**
     * @notice Mark an anomaly as resolved
     * @param metricId The ID of the metric
     * @param anomalyIndex The index of the anomaly in the metric's anomaly array
     */
    function resolveAnomaly(uint256 metricId, uint256 anomalyIndex) external onlyOwner {
        if (metricId >= metricCount) revert InvalidMetricId();
        if (anomalyIndex >= metricAnomalies[metricId].length) revert NoAnomalies();
        
        metricAnomalies[metricId][anomalyIndex].resolved = true;
        emit AnomalyResolved(metricId, anomalyIndex, block.timestamp);
    }
    
    /**
     * @notice Deactivate a metric (no longer monitored)
     * @param metricId The ID of the metric to deactivate
     */
    function deactivateMetric(uint256 metricId) external onlyOwner {
        if (metricId >= metricCount) revert InvalidMetricId();
        
        metrics[metricId].active = false;
    }
    
    /**
     * @notice Get metric information
     * @param metricId The ID of the metric
     * @return threshold The threshold value
     * @return currentValue The current metric value
     * @return lastUpdated Last update timestamp
     * @return active Whether the metric is active
     */
    function getMetric(uint256 metricId) external view returns (
        uint256 threshold,
        uint256 currentValue,
        uint256 lastUpdated,
        bool active
    ) {
        if (metricId >= metricCount) revert InvalidMetricId();
        
        Metric storage metric = metrics[metricId];
        return (
            metric.threshold,
            metric.currentValue,
            metric.lastUpdated,
            metric.active
        );
    }
    
    /**
     * @notice Get anomaly count for a metric
     * @param metricId The ID of the metric
     * @return count The number of anomalies for the metric
     */
    function getAnomalyCount(uint256 metricId) external view returns (uint256) {
        if (metricId >= metricCount) revert InvalidMetricId();
        
        return metricAnomalies[metricId].length;
    }
    
    /**
     * @notice Check if a value is anomalous for a given metric
     * @param metricId The ID of the metric
     * @param value The value to check
     * @return isAnomalous True if the value exceeds the threshold
     */
    function isAnomalous(uint256 metricId, uint256 value) external view returns (bool) {
        if (metricId >= metricCount) revert InvalidMetricId();
        
        return value > metrics[metricId].threshold;
    }
    
    /**
     * @notice Get system statistics
     * @return activeMetrics_ Number of active metrics
     * @return totalAnomalies_ Total anomalies detected
     */
    function getSystemStats() external view returns (uint256 activeMetrics_, uint256 totalAnomalies_) {
        uint256 activeCount = 0;
        
        for (uint256 i = 0; i < metricCount; i++) {
            if (metrics[i].active) {
                activeCount++;
            }
        }
        
        return (activeCount, totalAnomalies);
    }
}
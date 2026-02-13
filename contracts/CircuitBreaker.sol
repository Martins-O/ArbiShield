// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";

/**
 * @title CircuitBreaker
 * @notice Emergency stop mechanism for halting operations in ArbiShield
 * @dev Provides circuit breaker functionality with configurable trigger conditions
 */
contract CircuitBreaker is Ownable, Pausable {
    
    // Structs
    struct TriggerCondition {
        uint256 threshold;
        uint256 currentCount;
        uint256 windowStart;
        uint256 windowDuration; // in seconds
        bool active;
    }
    
    struct BreakerEvent {
        uint256 timestamp;
        string reason;
        address triggeredBy;
        uint256 triggerValue;
        bool manual;
    }
    
    // State variables
    mapping(uint256 => TriggerCondition) public triggerConditions;
    BreakerEvent[] public breakerHistory;
    
    uint256 public conditionCount;
    uint256 public autoRecoveryTime; // seconds until auto-recovery
    uint256 public lastTripTime;
    bool public autoRecoveryEnabled;
    
    // Events
    event ConditionAdded(uint256 indexed conditionId, uint256 threshold, uint256 windowDuration);
    event ConditionUpdated(uint256 indexed conditionId, uint256 threshold, uint256 windowDuration);
    event CircuitTripped(uint256 indexed conditionId, uint256 triggerValue, string reason);
    event CircuitReset(uint256 timestamp, string reason);
    event AutoRecoveryConfigured(uint256 recoveryTime);
    
    // Errors
    error InvalidConditionId();
    error ConditionNotActive();
    error CircuitNotTripped();
    error AlreadyTripped();
    error ThresholdZero();
    error WindowDurationZero();
    
    constructor() {
        _transferOwnership(msg.sender);
        conditionCount = 0;
        autoRecoveryTime = 3600;
        autoRecoveryEnabled = false;
    }
    
    /**
     * @notice Add a new trigger condition for the circuit breaker
     * @param threshold The threshold value that triggers the breaker
     * @param windowDuration Time window in seconds for threshold counting
     * @return conditionId The ID of the created condition
     */
    function addCondition(uint256 threshold, uint256 windowDuration) external onlyOwner returns (uint256) {
        if (threshold == 0) revert ThresholdZero();
        if (windowDuration == 0) revert WindowDurationZero();
        
        uint256 conditionId = conditionCount;
        triggerConditions[conditionId] = TriggerCondition({
            threshold: threshold,
            currentCount: 0,
            windowStart: block.timestamp,
            windowDuration: windowDuration,
            active: true
        });
        
        conditionCount++;
        emit ConditionAdded(conditionId, threshold, windowDuration);
        return conditionId;
    }
    
    /**
     * @notice Update an existing trigger condition
     * @param conditionId The ID of the condition to update
     * @param threshold The new threshold value
     * @param windowDuration The new window duration
     */
    function updateCondition(uint256 conditionId, uint256 threshold, uint256 windowDuration) external onlyOwner {
        if (conditionId >= conditionCount) revert InvalidConditionId();
        
        TriggerCondition storage condition = triggerConditions[conditionId];
        if (!condition.active) revert ConditionNotActive();
        
        if (threshold == 0) revert ThresholdZero();
        if (windowDuration == 0) revert WindowDurationZero();
        
        condition.threshold = threshold;
        condition.windowDuration = windowDuration;
        condition.windowStart = block.timestamp; // Reset window
        condition.currentCount = 0;
        
        emit ConditionUpdated(conditionId, threshold, windowDuration);
    }
    
    /**
     * @notice Trigger the circuit breaker manually
     * @param reason The reason for tripping the breaker
     */
    function tripManual(string memory reason) external onlyOwner {
        if (paused()) revert AlreadyTripped();
        
        _trip(0, 0, reason, true);
    }
    
    /**
     * @notice Check trigger conditions and trip if necessary
     * @param conditionId The ID of the condition to check
     * @param triggerValue The value that might trigger the breaker
     */
    function checkAndTrip(uint256 conditionId, uint256 triggerValue) external {
        if (paused()) revert AlreadyTripped();
        if (conditionId >= conditionCount) revert InvalidConditionId();
        
        TriggerCondition storage condition = triggerConditions[conditionId];
        if (!condition.active) revert ConditionNotActive();
        
        // Update time window if needed
        if (block.timestamp >= condition.windowStart + condition.windowDuration) {
            condition.windowStart = block.timestamp;
            condition.currentCount = 0;
        }
        
        condition.currentCount++;
        
        // Check if threshold is exceeded
        if (condition.currentCount >= condition.threshold) {
            _trip(conditionId, triggerValue, "Threshold exceeded", false);
        }
    }
    
    /**
     * @notice Reset the circuit breaker
     * @param reason The reason for resetting
     */
    function reset(string memory reason) external onlyOwner {
        if (!paused()) revert CircuitNotTripped();
        
        // Reset all condition counters
        for (uint256 i = 0; i < conditionCount; i++) {
            triggerConditions[i].currentCount = 0;
            triggerConditions[i].windowStart = block.timestamp;
        }
        
        lastTripTime = 0;
        _unpause();
        emit CircuitReset(block.timestamp, reason);
    }
    
    /**
     * @notice Configure auto-recovery
     * @param enabled Whether auto-recovery is enabled
     * @param recoveryTime Time in seconds for auto-recovery
     */
    function configureAutoRecovery(bool enabled, uint256 recoveryTime) external onlyOwner {
        autoRecoveryEnabled = enabled;
        autoRecoveryTime = recoveryTime;
        emit AutoRecoveryConfigured(recoveryTime);
    }
    
    /**
     * @notice Try auto-recovery if conditions are met
     */
    function attemptAutoRecovery() external {
        if (!paused()) revert CircuitNotTripped();
        if (!autoRecoveryEnabled) return;
        if (block.timestamp < lastTripTime + autoRecoveryTime) return;
        
        _unpause();
        emit CircuitReset(block.timestamp, "Auto-recovery");
    }
    
    /**
     * @notice Deactivate a trigger condition
     * @param conditionId The ID of the condition to deactivate
     */
    function deactivateCondition(uint256 conditionId) external onlyOwner {
        if (conditionId >= conditionCount) revert InvalidConditionId();
        
        triggerConditions[conditionId].active = false;
    }
    
    /**
     * @notice Get trigger condition details
     * @param conditionId The ID of the condition
     * @return threshold The threshold value
     * @return currentCount Current trigger count
     * @return windowStart Window start timestamp
     * @return windowDuration Window duration in seconds
     * @return active Whether the condition is active
     */
    function getCondition(uint256 conditionId) external view returns (
        uint256 threshold,
        uint256 currentCount,
        uint256 windowStart,
        uint256 windowDuration,
        bool active
    ) {
        if (conditionId >= conditionCount) revert InvalidConditionId();
        
        TriggerCondition storage condition = triggerConditions[conditionId];
        return (
            condition.threshold,
            condition.currentCount,
            condition.windowStart,
            condition.windowDuration,
            condition.active
        );
    }
    
    /**
     * @notice Check if auto-recovery is available
     * @return available True if auto-recovery can be triggered
     */
    function isAutoRecoveryAvailable() external view returns (bool) {
        return paused() && 
               autoRecoveryEnabled && 
               block.timestamp >= lastTripTime + autoRecoveryTime;
    }
    
    /**
     * @notice Get breaker history count
     * @return count Number of breaker events
     */
    function getHistoryCount() external view returns (uint256) {
        return breakerHistory.length;
    }
    
    /**
     * @notice Get system status
     * @return isTripped True if circuit is currently tripped
     * @return activeConditions Number of active conditions
     * @return lastTrip Last trip time (0 if never tripped)
     */
    function getStatus() external view returns (
        bool isTripped,
        uint256 activeConditions,
        uint256 lastTrip
    ) {
        uint256 activeCount = 0;
        
        for (uint256 i = 0; i < conditionCount; i++) {
            if (triggerConditions[i].active) {
                activeCount++;
            }
        }
        
        return (paused(), activeCount, lastTripTime);
    }
    
    // Internal function to trip the breaker
    function _trip(uint256 conditionId, uint256 triggerValue, string memory reason, bool manual) internal {
        lastTripTime = block.timestamp;
        _pause();
        
        BreakerEvent memory event_ = BreakerEvent({
            timestamp: block.timestamp,
            reason: reason,
            triggeredBy: manual ? msg.sender : address(0),
            triggerValue: triggerValue,
            manual: manual
        });
        
        breakerHistory.push(event_);
        emit CircuitTripped(conditionId, triggerValue, reason);
    }
}
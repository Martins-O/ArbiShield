// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title AlertRegistry
 * @notice Permanent record of security alerts for ArbiShield
 * @dev Maintains an immutable log of all security events and alerts
 */
contract AlertRegistry is Ownable, ReentrancyGuard {
    
    // Enums
    enum AlertSeverity {
        LOW,
        MEDIUM,
        HIGH,
        CRITICAL
    }
    
    enum AlertStatus {
        OPEN,
        INVESTIGATING,
        RESOLVED,
        FALSE_POSITIVE
    }
    
    // Structs
    struct Alert {
        uint256 id;
        bytes32 alertHash;
        address reporter;
        string title;
        string description;
        AlertSeverity severity;
        AlertStatus status;
        uint256 timestamp;
        uint256 resolvedAt;
        address resolver;
        string resolutionNotes;
        bool exists;
    }
    
    struct AlertFilter {
        address reporter;
        AlertSeverity severity;
        AlertStatus status;
        uint256 startTime;
        uint256 endTime;
    }
    
    // State variables
    mapping(uint256 => Alert) public alerts;
    mapping(bytes32 => uint256) public alertHashToId;
    mapping(address => uint256[]) public reporterAlerts;
    mapping(AlertSeverity => uint256[]) public severityAlerts;
    mapping(AlertStatus => uint256[]) public statusAlerts;
    
    uint256 public alertCount;
    uint256[] public allAlertIds;
    
    // Events
    event AlertRegistered(
        uint256 indexed id,
        bytes32 indexed alertHash,
        address indexed reporter,
        string title,
        AlertSeverity severity
    );
    event AlertUpdated(
        uint256 indexed id,
        AlertStatus oldStatus,
        AlertStatus newStatus,
        address indexed resolver
    );
    event AlertResolved(
        uint256 indexed id,
        address indexed resolver,
        string resolutionNotes
    );
    
    // Errors
    error AlertNotFound();
    error AlertAlreadyExists();
    error InvalidAlertId();
    error UnauthorizedResolver();
    error AlertAlreadyResolved();
    error EmptyTitle();
    error EmptyDescription();
    
    constructor() {
        _transferOwnership(msg.sender);
        alertCount = 0;
    }
    
    /**
     * @notice Register a new security alert
     * @param title Brief title of the alert
     * @param description Detailed description of the alert
     * @param severity Severity level of the alert
     * @return alertId The ID of the registered alert
     */
    function registerAlert(
        string memory title,
        string memory description,
        AlertSeverity severity
    ) external nonReentrant returns (uint256) {
        if (bytes(title).length == 0) revert EmptyTitle();
        if (bytes(description).length == 0) revert EmptyDescription();
        
        uint256 alertId = alertCount;
        bytes32 alertHash = keccak256(abi.encodePacked(
            msg.sender,
            title,
            description,
            severity,
            block.timestamp
        ));
        
        // Check for duplicates
        if (alertHashToId[alertHash] != 0) revert AlertAlreadyExists();
        
        // Create alert
        alerts[alertId] = Alert({
            id: alertId,
            alertHash: alertHash,
            reporter: msg.sender,
            title: title,
            description: description,
            severity: severity,
            status: AlertStatus.OPEN,
            timestamp: block.timestamp,
            resolvedAt: 0,
            resolver: address(0),
            resolutionNotes: "",
            exists: true
        });
        
        // Update mappings
        alertHashToId[alertHash] = alertId;
        reporterAlerts[msg.sender].push(alertId);
        severityAlerts[severity].push(alertId);
        statusAlerts[AlertStatus.OPEN].push(alertId);
        allAlertIds.push(alertId);
        
        alertCount++;
        
        emit AlertRegistered(alertId, alertHash, msg.sender, title, severity);
        return alertId;
    }
    
    /**
     * @notice Update alert status
     * @param alertId The ID of the alert to update
     * @param newStatus The new status for the alert
     */
    function updateAlertStatus(uint256 alertId, AlertStatus newStatus) external onlyOwner {
        if (alertId >= alertCount) revert InvalidAlertId();
        if (!alerts[alertId].exists) revert AlertNotFound();
        
        Alert storage alert = alerts[alertId];
        AlertStatus oldStatus = alert.status;
        
        if (oldStatus == AlertStatus.RESOLVED || oldStatus == AlertStatus.FALSE_POSITIVE) {
            revert AlertAlreadyResolved();
        }
        
        // Remove from old status mapping
        _removeFromStatusArray(alertId, oldStatus);
        
        // Update alert
        alert.status = newStatus;
        
        // Add to new status mapping
        statusAlerts[newStatus].push(alertId);
        
        if (newStatus == AlertStatus.RESOLVED || newStatus == AlertStatus.FALSE_POSITIVE) {
            alert.resolvedAt = block.timestamp;
            alert.resolver = msg.sender;
        }
        
        emit AlertUpdated(alertId, oldStatus, newStatus, msg.sender);
    }
    
    /**
     * @notice Resolve an alert with resolution notes
     * @param alertId The ID of the alert to resolve
     * @param resolutionNotes Notes about how the alert was resolved
     */
    function resolveAlert(
        uint256 alertId,
        string memory resolutionNotes
    ) external onlyOwner {
        if (alertId >= alertCount) revert InvalidAlertId();
        if (!alerts[alertId].exists) revert AlertNotFound();
        
        Alert storage alert = alerts[alertId];
        
        if (alert.status == AlertStatus.RESOLVED || alert.status == AlertStatus.FALSE_POSITIVE) {
            revert AlertAlreadyResolved();
        }
        
        // Remove from old status mapping
        _removeFromStatusArray(alertId, alert.status);
        
        // Update alert
        alert.status = AlertStatus.RESOLVED;
        alert.resolvedAt = block.timestamp;
        alert.resolver = msg.sender;
        alert.resolutionNotes = resolutionNotes;
        
        // Add to resolved status mapping
        statusAlerts[AlertStatus.RESOLVED].push(alertId);
        
        emit AlertResolved(alertId, msg.sender, resolutionNotes);
    }
    
    /**
     * @notice Get alert details
     * @param alertId The ID of the alert
     * @return alert Complete alert information
     */
    function getAlert(uint256 alertId) external view returns (Alert memory alert) {
        if (alertId >= alertCount) revert InvalidAlertId();
        if (!alerts[alertId].exists) revert AlertNotFound();
        
        return alerts[alertId];
    }
    
    /**
     * @notice Get alerts by reporter
     * @param reporter The address of the reporter
     * @return alertIds Array of alert IDs reported by the address
     */
    function getAlertsByReporter(address reporter) external view returns (uint256[] memory alertIds) {
        return reporterAlerts[reporter];
    }
    
    /**
     * @notice Get alerts by severity
     * @param severity The severity level
     * @return alertIds Array of alert IDs with the specified severity
     */
    function getAlertsBySeverity(AlertSeverity severity) external view returns (uint256[] memory alertIds) {
        return severityAlerts[severity];
    }
    
    /**
     * @notice Get alerts by status
     * @param status The alert status
     * @return alertIds Array of alert IDs with the specified status
     */
    function getAlertsByStatus(AlertStatus status) external view returns (uint256[] memory alertIds) {
        return statusAlerts[status];
    }
    
    /**
     * @notice Get all alert IDs
     * @return alertIds Array of all alert IDs
     */
    function getAllAlertIds() external view returns (uint256[] memory alertIds) {
        return allAlertIds;
    }
    
    /**
     * @notice Get system statistics
     * @return totalAlerts Total number of alerts
     * @return openAlerts Number of open alerts
     * @return resolvedAlerts Number of resolved alerts
     * @return criticalAlerts Number of critical alerts
     */
    function getSystemStats() external view returns (
        uint256 totalAlerts,
        uint256 openAlerts,
        uint256 resolvedAlerts,
        uint256 criticalAlerts
    ) {
        return (
            alertCount,
            statusAlerts[AlertStatus.OPEN].length,
            statusAlerts[AlertStatus.RESOLVED].length,
            severityAlerts[AlertSeverity.CRITICAL].length
        );
    }
    
    /**
     * @notice Check if an alert exists
     * @param alertId The ID of the alert to check
     * @return exists True if the alert exists
     */
    function alertExists(uint256 alertId) external view returns (bool) {
        return alertId < alertCount && alerts[alertId].exists;
    }
    
    /**
     * @notice Get alerts matching filter criteria
     * @param filter The filter criteria
     * @return alertIds Array of matching alert IDs
     */
    function getFilteredAlerts(AlertFilter memory filter) external view returns (uint256[] memory alertIds) {
        uint256[] memory tempIds = new uint256[](alertCount);
        uint256 count = 0;
        
        for (uint256 i = 0; i < alertCount; i++) {
            Alert memory alert = alerts[i];
            
            if (!alert.exists) continue;
            if (filter.reporter != address(0) && alert.reporter != filter.reporter) continue;
            if (filter.severity != AlertSeverity.LOW && alert.severity != filter.severity) continue;
            if (filter.status != AlertStatus.OPEN && alert.status != filter.status) continue;
            if (filter.startTime != 0 && alert.timestamp < filter.startTime) continue;
            if (filter.endTime != 0 && alert.timestamp > filter.endTime) continue;
            
            tempIds[count] = i;
            count++;
        }
        
        // Resize array to actual count
        alertIds = new uint256[](count);
        for (uint256 i = 0; i < count; i++) {
            alertIds[i] = tempIds[i];
        }
    }
    
    // Internal function to remove alert from status array
    function _removeFromStatusArray(uint256 alertId, AlertStatus status) internal {
        uint256[] storage statusArray = statusAlerts[status];
        uint256 length = statusArray.length;
        
        for (uint256 i = 0; i < length; i++) {
            if (statusArray[i] == alertId) {
                // Move last element to current position and pop
                statusArray[i] = statusArray[length - 1];
                statusArray.pop();
                break;
            }
        }
    }
}
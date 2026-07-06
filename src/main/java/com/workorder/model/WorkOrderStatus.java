package com.workorder.model;

/**
 * 代办事项处置状态。
 */
public enum WorkOrderStatus {

    NOT_STARTED("未处置"),
    IN_PROGRESS("处置中"),
    WAITING_REPLY("待回复"),
    COMPLETED("已完成");

    private final String displayName;

    WorkOrderStatus(String displayName) {
        this.displayName = displayName;
    }

    public String getDisplayName() {
        return displayName;
    }
}

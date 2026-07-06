package com.workorder.service;

import com.workorder.model.ProgressLog;
import com.workorder.model.WorkOrder;
import com.workorder.model.WorkOrderStatus;
import com.workorder.repository.ProgressLogRepository;
import com.workorder.repository.WorkOrderRepository;
import org.springframework.data.domain.Sort;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;
import org.springframework.util.StringUtils;

import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Objects;

/**
 * 代办事项业务服务。
 */
@Service
public class WorkOrderService {

    private static final Sort DEFAULT_SORT = Sort.by(Sort.Order.asc("priority"), Sort.Order.desc("updatedAt"));

    private final WorkOrderRepository workOrderRepository;
    private final ProgressLogRepository progressLogRepository;

    public WorkOrderService(WorkOrderRepository workOrderRepository,
                            ProgressLogRepository progressLogRepository) {
        this.workOrderRepository = workOrderRepository;
        this.progressLogRepository = progressLogRepository;
    }

    @Transactional
    public WorkOrder create(WorkOrder workOrder) {
        validateTitle(workOrder.getTitle());
        if (workOrder.getStatus() == null) {
            workOrder.setStatus(WorkOrderStatus.NOT_STARTED);
        }
        if (workOrder.getPriority() == null) {
            workOrder.setPriority(nextPriority());
        }
        WorkOrder saved = workOrderRepository.save(workOrder);
        appendWaitingReplyProgressLog(null, saved);
        return saved;
    }

    @Transactional
    public WorkOrder update(Long id, WorkOrder workOrder) {
        WorkOrder existing = getRequired(id);
        WorkOrder before = snapshotWaitingState(existing);
        validateTitle(workOrder.getTitle());
        existing.setTitle(workOrder.getTitle());
        existing.setDescription(workOrder.getDescription());
        existing.setStatus(workOrder.getStatus());
        existing.setWaitingFor(workOrder.getWaitingFor());
        existing.setWaitingReason(workOrder.getWaitingReason());
        existing.setDueDate(workOrder.getDueDate());
        WorkOrder saved = workOrderRepository.save(existing);
        appendWaitingReplyProgressLog(before, saved);
        return saved;
    }

    @Transactional
    public void delete(Long id) {
        getRequired(id);
        progressLogRepository.deleteByWorkOrderId(id);
        workOrderRepository.deleteById(id);
    }

    @Transactional(readOnly = true)
    public WorkOrder getRequired(Long id) {
        return workOrderRepository.findById(id)
                .orElseThrow(() -> new IllegalArgumentException("Work order not found: " + id));
    }

    @Transactional(readOnly = true)
    public List<WorkOrder> findAll(Sort sort) {
        return workOrderRepository.findAll(sort == null ? DEFAULT_SORT : sort);
    }

    @Transactional(readOnly = true)
    public List<WorkOrder> findByStatuses(List<WorkOrderStatus> statuses, boolean includeCompleted) {
        List<WorkOrderStatus> effectiveStatuses = new ArrayList<>();
        if (statuses == null || statuses.isEmpty()) {
            effectiveStatuses.addAll(Arrays.asList(WorkOrderStatus.values()));
        } else {
            effectiveStatuses.addAll(statuses);
        }
        if (!includeCompleted) {
            effectiveStatuses.remove(WorkOrderStatus.COMPLETED);
        }
        if (effectiveStatuses.isEmpty()) {
            return List.of();
        }
        return workOrderRepository.findByStatusIn(effectiveStatuses, DEFAULT_SORT);
    }

    @Transactional
    public void updatePriorities(List<Long> orderedIds) {
        if (orderedIds == null || orderedIds.isEmpty()) {
            return;
        }
        for (int i = 0; i < orderedIds.size(); i++) {
            WorkOrder workOrder = getRequired(orderedIds.get(i));
            workOrder.setPriority(i);
            workOrderRepository.save(workOrder);
        }
    }

    public boolean isOverdue(WorkOrder workOrder) {
        if (workOrder == null || workOrder.getDueDate() == null) {
            return false;
        }
        if (workOrder.getStatus() == WorkOrderStatus.COMPLETED) {
            return false;
        }
        return workOrder.getDueDate().isBefore(LocalDateTime.now());
    }

    private int nextPriority() {
        return workOrderRepository.findAll(Sort.by(Sort.Direction.DESC, "priority")).stream()
                .map(WorkOrder::getPriority)
                .findFirst()
                .map(p -> p + 1)
                .orElse(0);
    }

    private void validateTitle(String title) {
        if (!StringUtils.hasText(title)) {
            throw new IllegalArgumentException("Title is required");
        }
    }

    /**
     * 复制与待回复过程记录相关的字段，供更新前后对比。
     *
     * @param source 更新前的工单
     * @return 仅含状态与等待字段的快照
     */
    private WorkOrder snapshotWaitingState(WorkOrder source) {
        WorkOrder snapshot = new WorkOrder();
        snapshot.setStatus(source.getStatus());
        snapshot.setWaitingFor(source.getWaitingFor());
        snapshot.setWaitingReason(source.getWaitingReason());
        return snapshot;
    }

    /**
     * 状态变为待回复或等待信息变更时，将待回复过程写入处置时间线。
     *
     * @param before 更新前的工单；新建时为 null
     * @param after  已持久化的工单
     */
    private void appendWaitingReplyProgressLog(WorkOrder before, WorkOrder after) {
        if (after.getStatus() != WorkOrderStatus.WAITING_REPLY) {
            return;
        }
        boolean enteredWaiting = before == null || before.getStatus() != WorkOrderStatus.WAITING_REPLY;
        boolean waitingInfoChanged = before != null
                && before.getStatus() == WorkOrderStatus.WAITING_REPLY
                && (!Objects.equals(normalizeText(before.getWaitingFor()), normalizeText(after.getWaitingFor()))
                || !Objects.equals(normalizeText(before.getWaitingReason()), normalizeText(after.getWaitingReason())));
        if (!enteredWaiting && !waitingInfoChanged) {
            return;
        }
        ProgressLog log = new ProgressLog();
        log.setWorkOrderId(after.getId());
        log.setContent(formatWaitingReplyLog(after.getWaitingFor(), after.getWaitingReason()));
        progressLogRepository.save(log);
    }

    /**
     * 组装待回复过程记录文案。
     *
     * @param waitingFor    等待对象
     * @param waitingReason 等待原因
     * @return 写入时间线的过程内容
     */
    private String formatWaitingReplyLog(String waitingFor, String waitingReason) {
        StringBuilder content = new StringBuilder(WorkOrderStatus.WAITING_REPLY.getDisplayName());
        if (StringUtils.hasText(waitingFor)) {
            content.append("：等待 ").append(waitingFor.trim());
        }
        if (StringUtils.hasText(waitingReason)) {
            if (StringUtils.hasText(waitingFor)) {
                content.append("，");
            } else {
                content.append("：");
            }
            content.append("原因 ").append(waitingReason.trim());
        }
        return content.toString();
    }

    /**
     * 将空白文本规范为 null，便于比较等待字段是否变更。
     *
     * @param value 原始文本
     * @return 去空白后的文本；空白时返回 null
     */
    private String normalizeText(String value) {
        if (!StringUtils.hasText(value)) {
            return null;
        }
        return value.trim();
    }
}

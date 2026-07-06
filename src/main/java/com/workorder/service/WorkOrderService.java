package com.workorder.service;

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
        return workOrderRepository.save(workOrder);
    }

    @Transactional
    public WorkOrder update(Long id, WorkOrder workOrder) {
        WorkOrder existing = getRequired(id);
        validateTitle(workOrder.getTitle());
        existing.setTitle(workOrder.getTitle());
        existing.setDescription(workOrder.getDescription());
        existing.setStatus(workOrder.getStatus());
        existing.setWaitingFor(workOrder.getWaitingFor());
        existing.setWaitingReason(workOrder.getWaitingReason());
        existing.setDueDate(workOrder.getDueDate());
        return workOrderRepository.save(existing);
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
}

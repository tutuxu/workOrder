package com.workorder.service;

import com.workorder.model.WorkOrder;
import com.workorder.model.WorkOrderStatus;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.data.domain.Sort;

import java.time.LocalDateTime;
import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

@SpringBootTest
class WorkOrderServiceTest {

    @Autowired
    private WorkOrderService workOrderService;

    @Autowired
    private ProgressLogService progressLogService;

    @BeforeEach
    void cleanData() {
        workOrderService.findAll(Sort.unsorted()).forEach(item -> workOrderService.delete(item.getId()));
    }

    @Test
    void createAndFindAll() {
        WorkOrder created = workOrderService.create(buildWorkOrder("Task A"));
        assertThat(created.getId()).isNotNull();
        assertThat(workOrderService.findAll(Sort.by("priority"))).hasSize(1);
    }

    @Test
    void updateStatus() {
        WorkOrder created = workOrderService.create(buildWorkOrder("Task B"));
        created.setStatus(WorkOrderStatus.IN_PROGRESS);
        WorkOrder updated = workOrderService.update(created.getId(), created);
        assertThat(updated.getStatus()).isEqualTo(WorkOrderStatus.IN_PROGRESS);
    }

    @Test
    void filterByStatusesAndHideCompleted() {
        WorkOrder open = workOrderService.create(buildWorkOrder("Open"));
        WorkOrder done = workOrderService.create(buildWorkOrder("Done"));
        done.setStatus(WorkOrderStatus.COMPLETED);
        workOrderService.update(done.getId(), done);

        List<WorkOrder> withoutCompleted = workOrderService.findByStatuses(List.of(), false);
        assertThat(withoutCompleted).extracting(WorkOrder::getTitle).containsExactly("Open");

        List<WorkOrder> onlyCompleted = workOrderService.findByStatuses(
                List.of(WorkOrderStatus.COMPLETED), true);
        assertThat(onlyCompleted).extracting(WorkOrder::getTitle).containsExactly("Done");
    }

    @Test
    void updatePriorities() {
        WorkOrder first = workOrderService.create(buildWorkOrder("First"));
        WorkOrder second = workOrderService.create(buildWorkOrder("Second"));
        workOrderService.updatePriorities(List.of(second.getId(), first.getId()));

        List<WorkOrder> ordered = workOrderService.findAll(Sort.by("priority"));
        assertThat(ordered.get(0).getId()).isEqualTo(second.getId());
        assertThat(ordered.get(1).getId()).isEqualTo(first.getId());
    }

    @Test
    void isOverdue() {
        WorkOrder overdue = buildWorkOrder("Overdue");
        overdue.setDueDate(LocalDateTime.now().minusHours(1));
        overdue = workOrderService.create(overdue);
        assertThat(workOrderService.isOverdue(overdue)).isTrue();

        overdue.setStatus(WorkOrderStatus.COMPLETED);
        workOrderService.update(overdue.getId(), overdue);
        assertThat(workOrderService.isOverdue(overdue)).isFalse();
    }

    @Test
    void addProgressLog() {
        WorkOrder created = workOrderService.create(buildWorkOrder("With log"));
        progressLogService.addLog(created.getId(), "Started investigation");
        assertThat(progressLogService.findByWorkOrderId(created.getId())).hasSize(1);
    }

    @Test
    void titleRequired() {
        WorkOrder invalid = buildWorkOrder(" ");
        assertThatThrownBy(() -> workOrderService.create(invalid))
                .isInstanceOf(IllegalArgumentException.class);
    }

    private WorkOrder buildWorkOrder(String title) {
        WorkOrder workOrder = new WorkOrder();
        workOrder.setTitle(title);
        return workOrder;
    }
}

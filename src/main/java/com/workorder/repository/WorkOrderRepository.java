package com.workorder.repository;

import com.workorder.model.WorkOrder;
import com.workorder.model.WorkOrderStatus;
import org.springframework.data.domain.Sort;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;

/**
 * 代办事项持久化接口。
 */
public interface WorkOrderRepository extends JpaRepository<WorkOrder, Long> {

    List<WorkOrder> findByStatusIn(List<WorkOrderStatus> statuses, Sort sort);
}

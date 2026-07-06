package com.workorder.repository;

import com.workorder.model.ProgressLog;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.List;

/**
 * 处置过程记录持久化接口。
 */
public interface ProgressLogRepository extends JpaRepository<ProgressLog, Long> {

    List<ProgressLog> findByWorkOrderIdOrderByCreatedAtDesc(Long workOrderId);

    void deleteByWorkOrderId(Long workOrderId);
}

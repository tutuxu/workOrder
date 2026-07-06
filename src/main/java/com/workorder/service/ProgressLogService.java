package com.workorder.service;

import com.workorder.model.ProgressLog;
import com.workorder.repository.ProgressLogRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;
import org.springframework.util.StringUtils;

import java.util.List;

/**
 * 处置过程记录业务服务。
 */
@Service
public class ProgressLogService {

    private final ProgressLogRepository progressLogRepository;
    private final WorkOrderService workOrderService;

    public ProgressLogService(ProgressLogRepository progressLogRepository,
                              WorkOrderService workOrderService) {
        this.progressLogRepository = progressLogRepository;
        this.workOrderService = workOrderService;
    }

    @Transactional
    public ProgressLog addLog(Long workOrderId, String content) {
        if (!StringUtils.hasText(content)) {
            throw new IllegalArgumentException("Progress content is required");
        }
        workOrderService.getRequired(workOrderId);
        ProgressLog log = new ProgressLog();
        log.setWorkOrderId(workOrderId);
        log.setContent(content.trim());
        return progressLogRepository.save(log);
    }

    @Transactional(readOnly = true)
    public List<ProgressLog> findByWorkOrderId(Long workOrderId) {
        return progressLogRepository.findByWorkOrderIdOrderByCreatedAtDesc(workOrderId);
    }
}

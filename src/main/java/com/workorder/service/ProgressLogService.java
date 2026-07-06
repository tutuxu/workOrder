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

    /**
     * 更新指定工单的处置过程内容。
     *
     * @param logId       过程记录 ID
     * @param workOrderId 所属工单 ID
     * @param content     新内容
     * @return 更新后的记录
     */
    @Transactional
    public ProgressLog updateLog(Long logId, Long workOrderId, String content) {
        if (!StringUtils.hasText(content)) {
            throw new IllegalArgumentException("Progress content is required");
        }
        ProgressLog log = getRequiredLog(logId, workOrderId);
        log.setContent(content.trim());
        return progressLogRepository.save(log);
    }

    /**
     * 删除指定工单的处置过程记录。
     *
     * @param logId       过程记录 ID
     * @param workOrderId 所属工单 ID
     */
    @Transactional
    public void deleteLog(Long logId, Long workOrderId) {
        ProgressLog log = getRequiredLog(logId, workOrderId);
        progressLogRepository.delete(log);
    }

    @Transactional(readOnly = true)
    public List<ProgressLog> findByWorkOrderId(Long workOrderId) {
        return progressLogRepository.findByWorkOrderIdOrderByCreatedAtDesc(workOrderId);
    }

    /**
     * 获取属于指定工单的过程记录，不存在或不匹配时抛异常。
     *
     * @param logId       过程记录 ID
     * @param workOrderId 所属工单 ID
     * @return 过程记录
     */
    private ProgressLog getRequiredLog(Long logId, Long workOrderId) {
        ProgressLog log = progressLogRepository.findById(logId)
                .orElseThrow(() -> new IllegalArgumentException("Progress log not found: " + logId));
        if (!workOrderId.equals(log.getWorkOrderId())) {
            throw new IllegalArgumentException("Progress log does not belong to work order: " + workOrderId);
        }
        return log;
    }
}

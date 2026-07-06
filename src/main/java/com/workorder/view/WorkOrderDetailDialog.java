package com.workorder.view;

import com.vaadin.flow.component.button.Button;
import com.vaadin.flow.component.button.ButtonVariant;
import com.vaadin.flow.component.confirmdialog.ConfirmDialog;
import com.vaadin.flow.component.datetimepicker.DateTimePicker;
import com.vaadin.flow.component.dialog.Dialog;
import com.vaadin.flow.component.formlayout.FormLayout;
import com.vaadin.flow.component.html.H3;
import com.vaadin.flow.component.html.Span;
import com.vaadin.flow.component.notification.Notification;
import com.vaadin.flow.component.orderedlayout.FlexComponent.Alignment;
import com.vaadin.flow.component.orderedlayout.FlexComponent.JustifyContentMode;
import com.vaadin.flow.component.orderedlayout.HorizontalLayout;
import com.vaadin.flow.component.orderedlayout.VerticalLayout;
import com.vaadin.flow.component.radiobutton.RadioButtonGroup;
import com.vaadin.flow.component.textfield.TextArea;
import com.vaadin.flow.component.textfield.TextField;
import com.workorder.model.ProgressLog;
import com.workorder.model.WorkOrder;
import com.workorder.model.WorkOrderStatus;
import com.workorder.service.ProgressLogService;
import com.workorder.service.WorkOrderService;

import java.time.format.DateTimeFormatter;
import java.util.Arrays;

import org.springframework.util.StringUtils;

/**
 * 代办详情/编辑对话框。
 */
public class WorkOrderDetailDialog extends Dialog {

    private static final DateTimeFormatter DATE_TIME_FORMATTER = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm");

    private final WorkOrderService workOrderService;
    private final ProgressLogService progressLogService;
    private final Runnable onSaved;

    private WorkOrder workOrder;
    private final TextField titleField = new TextField("标题");
    private final TextArea descriptionField = new TextArea("描述");
    private final RadioButtonGroup<WorkOrderStatus> statusGroup = new RadioButtonGroup<>();
    private final DateTimePicker dueDatePicker = new DateTimePicker("计划完成时间");
    private final TextField waitingForField = new TextField("等待对象");
    private final TextField waitingReasonField = new TextField("等待原因");
    private final VerticalLayout timelineLayout = new VerticalLayout();
    private final TextField progressInput = new TextField("追加过程");
    private final Button addProgressButton = new Button("追加");
    private final Button cancelEditProgressButton = new Button("取消");
    private final Button saveButton = new Button("保存");
    private final Button deleteButton = new Button("删除");

    private Long editingLogId;

    public WorkOrderDetailDialog(WorkOrder workOrder,
                                 WorkOrderService workOrderService,
                                 ProgressLogService progressLogService,
                                 Runnable onSaved) {
        this.workOrder = workOrder;
        this.workOrderService = workOrderService;
        this.progressLogService = progressLogService;
        this.onSaved = onSaved;

        setHeaderTitle(workOrder.getId() == null ? "新建代办" : "编辑代办");
        setWidth("640px");
        setDraggable(true);

        configureFields();
        bindWorkOrder(workOrder);
        add(buildContent());
    }

    private void configureFields() {
        titleField.setRequired(true);
        titleField.setWidthFull();
        descriptionField.setWidthFull();
        descriptionField.setHeight("120px");

        statusGroup.setLabel("状态");
        statusGroup.setItems(Arrays.asList(WorkOrderStatus.values()));
        statusGroup.setItemLabelGenerator(WorkOrderStatus::getDisplayName);
        statusGroup.addValueChangeListener(event -> updateWaitingFieldsVisibility());

        waitingForField.setWidthFull();
        waitingReasonField.setWidthFull();

        progressInput.setWidthFull();
        addProgressButton.addClickListener(event -> saveProgress());
        cancelEditProgressButton.addThemeVariants(ButtonVariant.LUMO_TERTIARY);
        cancelEditProgressButton.setVisible(false);
        cancelEditProgressButton.addClickListener(event -> clearEditMode());

        saveButton.addThemeVariants(ButtonVariant.LUMO_PRIMARY);
        saveButton.addClickListener(event -> save());

        deleteButton.addThemeVariants(ButtonVariant.LUMO_ERROR);
        deleteButton.setVisible(workOrder.getId() != null);
        deleteButton.addClickListener(event -> confirmDelete());

        timelineLayout.setPadding(false);
        timelineLayout.setSpacing(true);
    }

    private VerticalLayout buildContent() {
        FormLayout formLayout = new FormLayout(titleField, descriptionField, statusGroup,
                dueDatePicker, waitingForField, waitingReasonField);
        formLayout.setResponsiveSteps(new FormLayout.ResponsiveStep("0", 1));

        H3 timelineTitle = new H3("处置过程");
        HorizontalLayout progressBar = new HorizontalLayout(progressInput, addProgressButton, cancelEditProgressButton);
        progressBar.setAlignItems(Alignment.END);
        progressBar.setWidthFull();
        progressBar.expand(progressInput);

        HorizontalLayout actions = new HorizontalLayout(saveButton, deleteButton);
        actions.setWidthFull();
        actions.setJustifyContentMode(JustifyContentMode.END);

        VerticalLayout content = new VerticalLayout(formLayout, timelineTitle, timelineLayout,
                progressBar, actions);
        content.setPadding(false);
        content.setSpacing(true);
        return content;
    }

    private void bindWorkOrder(WorkOrder item) {
        titleField.setValue(item.getTitle() == null ? "" : item.getTitle());
        descriptionField.setValue(item.getDescription() == null ? "" : item.getDescription());
        statusGroup.setValue(item.getStatus() == null ? WorkOrderStatus.NOT_STARTED : item.getStatus());
        dueDatePicker.setValue(item.getDueDate());
        waitingForField.setValue(item.getWaitingFor() == null ? "" : item.getWaitingFor());
        waitingReasonField.setValue(item.getWaitingReason() == null ? "" : item.getWaitingReason());
        updateWaitingFieldsVisibility();
        refreshTimeline();
    }

    private void updateWaitingFieldsVisibility() {
        boolean waiting = statusGroup.getValue() == WorkOrderStatus.WAITING_REPLY;
        waitingForField.setVisible(waiting);
        waitingReasonField.setVisible(waiting);
    }

    private void refreshTimeline() {
        timelineLayout.removeAll();
        if (workOrder.getId() == null) {
            timelineLayout.add(new Span("保存后可追加处置过程"));
            return;
        }
        for (ProgressLog log : progressLogService.findByWorkOrderId(workOrder.getId())) {
            timelineLayout.add(createTimelineEntry(log));
        }
        if (timelineLayout.getComponentCount() == 0) {
            timelineLayout.add(new Span("暂无过程记录"));
        }
    }

    /**
     * 构建单条处置过程行，含编辑与删除操作。
     *
     * @param log 过程记录
     * @return 时间线行布局
     */
    private HorizontalLayout createTimelineEntry(ProgressLog log) {
        Span entry = new Span(log.getCreatedAt().format(DATE_TIME_FORMATTER) + " — " + log.getContent());
        entry.getStyle().set("flex-grow", "1");

        Button editButton = new Button("编辑");
        editButton.addThemeVariants(ButtonVariant.LUMO_TERTIARY_INLINE);
        editButton.addClickListener(event -> startEdit(log));

        Button deleteProgressButton = new Button("删除");
        deleteProgressButton.addThemeVariants(ButtonVariant.LUMO_TERTIARY_INLINE, ButtonVariant.LUMO_ERROR);
        deleteProgressButton.addClickListener(event -> confirmDeleteProgress(log));

        HorizontalLayout row = new HorizontalLayout(entry, editButton, deleteProgressButton);
        row.setWidthFull();
        row.setAlignItems(Alignment.CENTER);
        return row;
    }

    /**
     * 进入过程编辑模式，将内容加载到底部输入框。
     *
     * @param log 待编辑的过程记录
     */
    private void startEdit(ProgressLog log) {
        editingLogId = log.getId();
        progressInput.setValue(log.getContent());
        addProgressButton.setText("保存修改");
        cancelEditProgressButton.setVisible(true);
    }

    /**
     * 退出过程编辑模式并清空底部输入框。
     */
    private void clearEditMode() {
        editingLogId = null;
        progressInput.clear();
        addProgressButton.setText("追加");
        cancelEditProgressButton.setVisible(false);
    }

    /**
     * 追加新过程或保存正在编辑的过程。
     */
    private void saveProgress() {
        if (workOrder.getId() == null) {
            Notification.show("请先保存代办事项");
            return;
        }
        try {
            if (editingLogId != null) {
                progressLogService.updateLog(editingLogId, workOrder.getId(), progressInput.getValue());
                clearEditMode();
            } else {
                progressLogService.addLog(workOrder.getId(), progressInput.getValue());
                progressInput.clear();
            }
            refreshTimeline();
        } catch (IllegalArgumentException ex) {
            Notification.show(ex.getMessage());
        }
    }

    /**
     * 确认后删除单条处置过程记录。
     *
     * @param log 待删除的过程记录
     */
    private void confirmDeleteProgress(ProgressLog log) {
        ConfirmDialog dialog = new ConfirmDialog();
        dialog.setHeader("确认删除");
        dialog.setText("确定删除该过程记录吗？");
        dialog.setCancelable(true);
        dialog.setConfirmText("删除");
        dialog.setConfirmButtonTheme("error primary");
        dialog.addConfirmListener(event -> {
            progressLogService.deleteLog(log.getId(), workOrder.getId());
            if (log.getId().equals(editingLogId)) {
                clearEditMode();
            }
            refreshTimeline();
        });
        dialog.open();
    }

    private void save() {
        workOrder.setTitle(titleField.getValue());
        workOrder.setDescription(descriptionField.getValue());
        workOrder.setStatus(statusGroup.getValue());
        workOrder.setDueDate(dueDatePicker.getValue());
        workOrder.setWaitingFor(waitingForField.getValue());
        workOrder.setWaitingReason(waitingReasonField.getValue());

        try {
            if (workOrder.getId() == null) {
                workOrder = workOrderService.create(workOrder);
            } else {
                workOrder = workOrderService.update(workOrder.getId(), workOrder);
            }
            deleteButton.setVisible(true);
            flushPendingProgressInput();
            refreshTimeline();
            onSaved.run();
            Notification.show("已保存");
        } catch (IllegalArgumentException ex) {
            Notification.show(ex.getMessage());
        }
    }

    /**
     * 保存工单后，将底部输入框中尚未提交的过程内容写入时间线。
     */
    private void flushPendingProgressInput() {
        String pendingContent = progressInput.getValue();
        if (!StringUtils.hasText(pendingContent)) {
            return;
        }
        try {
            if (editingLogId != null) {
                progressLogService.updateLog(editingLogId, workOrder.getId(), pendingContent);
                clearEditMode();
            } else {
                progressLogService.addLog(workOrder.getId(), pendingContent);
                progressInput.clear();
            }
        } catch (IllegalArgumentException ex) {
            Notification.show(ex.getMessage());
        }
    }

    private void confirmDelete() {
        ConfirmDialog dialog = new ConfirmDialog();
        dialog.setHeader("确认删除");
        dialog.setText("确定删除该代办事项吗？");
        dialog.setCancelable(true);
        dialog.setConfirmText("删除");
        dialog.setConfirmButtonTheme("error primary");
        dialog.addConfirmListener(event -> {
            workOrderService.delete(workOrder.getId());
            onSaved.run();
            close();
        });
        dialog.open();
    }
}

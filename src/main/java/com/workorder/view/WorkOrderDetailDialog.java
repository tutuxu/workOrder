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
import com.vaadin.flow.component.select.Select;
import com.vaadin.flow.component.textfield.TextArea;
import com.vaadin.flow.component.textfield.TextField;
import com.workorder.model.ProgressLog;
import com.workorder.model.WorkOrder;
import com.workorder.model.WorkOrderStatus;
import com.workorder.service.ProgressLogService;
import com.workorder.service.WorkOrderService;

import java.time.format.DateTimeFormatter;
import java.util.Arrays;

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
    private final Select<WorkOrderStatus> statusSelect = new Select<>();
    private final DateTimePicker dueDatePicker = new DateTimePicker("计划完成时间");
    private final TextField waitingForField = new TextField("等待对象");
    private final TextField waitingReasonField = new TextField("等待原因");
    private final VerticalLayout timelineLayout = new VerticalLayout();
    private final TextField progressInput = new TextField("追加过程");
    private final Button addProgressButton = new Button("追加");
    private final Button saveButton = new Button("保存");
    private final Button deleteButton = new Button("删除");

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

        statusSelect.setLabel("状态");
        statusSelect.setItems(Arrays.asList(WorkOrderStatus.values()));
        statusSelect.setItemLabelGenerator(WorkOrderStatus::getDisplayName);
        statusSelect.addValueChangeListener(event -> updateWaitingFieldsVisibility());

        waitingForField.setWidthFull();
        waitingReasonField.setWidthFull();

        progressInput.setWidthFull();
        addProgressButton.addClickListener(event -> appendProgress());

        saveButton.addThemeVariants(ButtonVariant.LUMO_PRIMARY);
        saveButton.addClickListener(event -> save());

        deleteButton.addThemeVariants(ButtonVariant.LUMO_ERROR);
        deleteButton.setVisible(workOrder.getId() != null);
        deleteButton.addClickListener(event -> confirmDelete());

        timelineLayout.setPadding(false);
        timelineLayout.setSpacing(true);
    }

    private VerticalLayout buildContent() {
        FormLayout formLayout = new FormLayout(titleField, descriptionField, statusSelect,
                dueDatePicker, waitingForField, waitingReasonField);
        formLayout.setResponsiveSteps(new FormLayout.ResponsiveStep("0", 1));

        H3 timelineTitle = new H3("处置过程");
        HorizontalLayout progressBar = new HorizontalLayout(progressInput, addProgressButton);
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
        statusSelect.setValue(item.getStatus() == null ? WorkOrderStatus.NOT_STARTED : item.getStatus());
        dueDatePicker.setValue(item.getDueDate());
        waitingForField.setValue(item.getWaitingFor() == null ? "" : item.getWaitingFor());
        waitingReasonField.setValue(item.getWaitingReason() == null ? "" : item.getWaitingReason());
        updateWaitingFieldsVisibility();
        refreshTimeline();
    }

    private void updateWaitingFieldsVisibility() {
        boolean waiting = statusSelect.getValue() == WorkOrderStatus.WAITING_REPLY;
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
            Span entry = new Span(log.getCreatedAt().format(DATE_TIME_FORMATTER) + " — " + log.getContent());
            timelineLayout.add(entry);
        }
        if (timelineLayout.getComponentCount() == 0) {
            timelineLayout.add(new Span("暂无过程记录"));
        }
    }

    private void appendProgress() {
        if (workOrder.getId() == null) {
            Notification.show("请先保存代办事项");
            return;
        }
        try {
            progressLogService.addLog(workOrder.getId(), progressInput.getValue());
            progressInput.clear();
            refreshTimeline();
        } catch (IllegalArgumentException ex) {
            Notification.show(ex.getMessage());
        }
    }

    private void save() {
        workOrder.setTitle(titleField.getValue());
        workOrder.setDescription(descriptionField.getValue());
        workOrder.setStatus(statusSelect.getValue());
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
            refreshTimeline();
            onSaved.run();
            Notification.show("已保存");
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

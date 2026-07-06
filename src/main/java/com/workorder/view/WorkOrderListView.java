package com.workorder.view;

import com.vaadin.flow.component.button.Button;
import com.vaadin.flow.component.button.ButtonVariant;
import com.vaadin.flow.component.checkbox.Checkbox;
import com.vaadin.flow.component.html.Span;
import com.vaadin.flow.component.grid.Grid;
import com.vaadin.flow.component.grid.dnd.GridDragStartEvent;
import com.vaadin.flow.component.grid.dnd.GridDropLocation;
import com.vaadin.flow.component.grid.dnd.GridDropMode;
import com.vaadin.flow.component.orderedlayout.FlexComponent.Alignment;
import com.vaadin.flow.component.orderedlayout.HorizontalLayout;
import com.vaadin.flow.component.orderedlayout.VerticalLayout;
import com.vaadin.flow.data.renderer.TextRenderer;
import com.vaadin.flow.router.PageTitle;
import com.vaadin.flow.router.Route;
import com.vaadin.flow.theme.lumo.LumoUtility;
import com.workorder.model.WorkOrder;
import com.workorder.model.WorkOrderStatus;
import com.workorder.service.ProgressLogService;
import com.workorder.service.WorkOrderService;
import org.springframework.beans.factory.annotation.Autowired;

import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.List;
import java.util.EnumMap;
import java.util.Map;
import java.util.stream.Collectors;

/**
 * 代办列表页：筛选、拖拽排序、详情编辑入口。
 */
@Route(value = "", layout = MainLayout.class)
@PageTitle("workOrder")
public class WorkOrderListView extends VerticalLayout {

    private static final DateTimeFormatter DATE_TIME_FORMATTER = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm");
    private static final String OVERDUE_ROW_CLASS = "overdue-row";

    private final WorkOrderService workOrderService;
    private final ProgressLogService progressLogService;

    private final Grid<WorkOrder> grid = new Grid<>(WorkOrder.class, false);
    private final Map<WorkOrderStatus, Checkbox> statusFilterCheckboxes = new EnumMap<>(WorkOrderStatus.class);
    private final HorizontalLayout statusFilterLayout = new HorizontalLayout();
    private final Checkbox showCompletedCheckbox = new Checkbox("显示已完成", false);
    private final Button createButton = new Button("新建");
    private WorkOrder draggedItem;

    @Autowired
    public WorkOrderListView(WorkOrderService workOrderService,
                             ProgressLogService progressLogService) {
        this.workOrderService = workOrderService;
        this.progressLogService = progressLogService;
        setSizeFull();
        addClassNames(LumoUtility.Padding.MEDIUM);

        configureToolbar();
        configureGrid();
        refreshGrid();

        add(createToolbar(), grid);
        expand(grid);
    }

    private void configureToolbar() {
        statusFilterLayout.add(new Span("状态筛选"));
        statusFilterLayout.setAlignItems(Alignment.CENTER);
        for (WorkOrderStatus status : WorkOrderStatus.values()) {
            Checkbox checkbox = new Checkbox(status.getDisplayName());
            checkbox.addValueChangeListener(event -> refreshGrid());
            statusFilterCheckboxes.put(status, checkbox);
            statusFilterLayout.add(checkbox);
        }

        showCompletedCheckbox.addValueChangeListener(event -> refreshGrid());

        createButton.addThemeVariants(ButtonVariant.LUMO_PRIMARY);
        createButton.addClickListener(event -> openDetailDialog(new WorkOrder()));
    }

    private HorizontalLayout createToolbar() {
        HorizontalLayout toolbar = new HorizontalLayout(createButton, statusFilterLayout, showCompletedCheckbox);
        toolbar.setWidthFull();
        toolbar.setAlignItems(Alignment.CENTER);
        toolbar.setDefaultVerticalComponentAlignment(Alignment.CENTER);
        return toolbar;
    }

    private void configureGrid() {
        grid.addColumn(WorkOrder::getTitle).setHeader("标题").setFlexGrow(2);
        grid.addColumn(new TextRenderer<>(item ->
                item.getStatus() == null ? "" : item.getStatus().getDisplayName()))
                .setHeader("状态").setFlexGrow(1);
        grid.addColumn(new TextRenderer<>(item ->
                item.getDueDate() == null ? "" : item.getDueDate().format(DATE_TIME_FORMATTER)))
                .setHeader("计划完成时间").setFlexGrow(1);
        grid.addColumn(new TextRenderer<>(item ->
                item.getUpdatedAt() == null ? "" : item.getUpdatedAt().format(DATE_TIME_FORMATTER)))
                .setHeader("最后更新").setFlexGrow(1);

        grid.setClassNameGenerator(item ->
                workOrderService.isOverdue(item) ? OVERDUE_ROW_CLASS : null);

        grid.addItemClickListener(event -> openDetailDialog(event.getItem()));

        grid.setRowsDraggable(true);
        grid.setDropMode(GridDropMode.BETWEEN);
        grid.addDragStartListener(this::onDragStart);
        grid.addDropListener(event -> {
            WorkOrder dropped = event.getDropTargetItem().orElse(null);
            if (dropped == null || draggedItem == null || draggedItem.getId().equals(dropped.getId())) {
                return;
            }
            List<WorkOrder> items = new ArrayList<>(grid.getListDataView().getItems().toList());
            items.remove(draggedItem);
            int dropIndex = items.indexOf(dropped);
            if (event.getDropLocation() == GridDropLocation.BELOW) {
                dropIndex++;
            }
            items.add(dropIndex, draggedItem);
            List<Long> orderedIds = items.stream().map(WorkOrder::getId).collect(Collectors.toList());
            workOrderService.updatePriorities(orderedIds);
            draggedItem = null;
            refreshGrid();
        });

        grid.setSizeFull();
    }

    private void onDragStart(GridDragStartEvent<WorkOrder> event) {
        draggedItem = event.getDraggedItems().isEmpty() ? null : event.getDraggedItems().get(0);
    }

    private void refreshGrid() {
        List<WorkOrderStatus> statusList = statusFilterCheckboxes.entrySet().stream()
                .filter(entry -> Boolean.TRUE.equals(entry.getValue().getValue()))
                .map(Map.Entry::getKey)
                .collect(Collectors.toList());
        List<WorkOrder> items = workOrderService.findByStatuses(statusList, showCompletedCheckbox.getValue());
        grid.setItems(items);
    }

    private void openDetailDialog(WorkOrder workOrder) {
        WorkOrderDetailDialog dialog = new WorkOrderDetailDialog(
                workOrder,
                workOrderService,
                progressLogService,
                this::refreshGrid
        );
        dialog.open();
    }
}

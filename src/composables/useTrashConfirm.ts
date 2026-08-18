import { h } from "vue";
import { NButton, useDialog } from "naive-ui";

export function useTrashConfirm() {
  const dialog = useDialog();

  function confirmPermanentDelete(options: {
    count: number;
    onConfirm: () => void | Promise<void>;
  }) {
    const n = options.count;
    dialog.error({
      title: "确认彻底删除",
      content: `此操作不可恢复，确定彻底删除 ${n} 条？`,
      positiveText: "彻底删除",
      negativeText: "取消",
      onPositiveClick: () => options.onConfirm(),
    });
  }

  function confirmMoveToTrash(options: {
    count: number;
    singular?: boolean;
    onTrash: () => void | Promise<void>;
    onPermanent: () => void | Promise<void>;
  }) {
    const n = options.count;
    const content = options.singular
      ? "确定将该代办事项移入回收站？可在回收站还原。"
      : `确定将选中的 ${n} 条移入回收站？可在回收站还原。`;

    const d = dialog.create({
      title: "确认删除",
      content,
      closable: true,
      maskClosable: true,
      action: () =>
        h("div", { style: "display:flex;gap:8px;justify-content:flex-end;flex-wrap:wrap" }, [
          h(NButton, { onClick: () => d.destroy() }, { default: () => "取消" }),
          h(
            NButton,
            {
              type: "error",
              ghost: true,
              onClick: () => {
                d.destroy();
                confirmPermanentDelete({
                  count: options.count,
                  onConfirm: options.onPermanent,
                });
              },
            },
            { default: () => "彻底删除" },
          ),
          h(
            NButton,
            {
              type: "primary",
              onClick: () => {
                d.destroy();
                void options.onTrash();
              },
            },
            { default: () => "移入回收站" },
          ),
        ]),
    });
  }

  return { confirmMoveToTrash, confirmPermanentDelete };
}

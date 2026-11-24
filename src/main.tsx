import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import { initTheme } from "./theme";
import { applySkin, loadSkin } from "./skins";
import { initI18n } from "./i18n";

import { PLATFORM } from "./platform";

initI18n();
initTheme();
applySkin(loadSkin());
// 平台标记（CSS 按平台差异化），首屏即可用，无异步闪烁
document.documentElement.dataset.platform = PLATFORM;

// ---------- 拖拽减负 ----------
// 按住标题栏（data-tauri-drag-region）期间给 <html> 挂 dragging 类：
// CSS 侧冻结全部 transition / 暂停全部 animation（见 index.css），拖动
// 帧率不再与 VU 表、律动层、卡片过渡抢同一份渲染预算——"拖拽不跟手"
// 的主因之一。OS 接管窗口拖动后 mouseup 不一定派发回页面，定时器兜底
let draggingTimer: number | undefined;
document.addEventListener(
  "mousedown",
  (e) => {
    if (e.button !== 0) return;
    const el = e.target as Element | null;
    if (!el?.closest?.("[data-tauri-drag-region]")) return;
    document.documentElement.classList.add("dragging");
    if (draggingTimer != null) window.clearTimeout(draggingTimer);
    draggingTimer = window.setTimeout(() => {
      document.documentElement.classList.remove("dragging");
      draggingTimer = undefined;
    }, 2500);
  },
  true
);
document.addEventListener(
  "mouseup",
  () => {
    if (draggingTimer != null) {
      window.clearTimeout(draggingTimer);
      draggingTimer = undefined;
    }
    document.documentElement.classList.remove("dragging");
  },
  true
);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

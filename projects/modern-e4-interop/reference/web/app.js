"use strict";
const byId = id => document.getElementById(id);
const output = byId("result");
function integer(text) {
    const value = text.trim();
    if (!/^[+-]?\d+$/.test(value)) throw Error("请输入整数");
    const number = Number(value);
    if (!Number.isSafeInteger(number) || Math.abs(number) > 100000) throw Error("超出允许范围");
    return number;
}
(async function () {
    try {
        const response = await fetch("e4_signal_bridge.wasm");
        if (!response.ok) throw Error("WASM 加载失败");
        const { instance } = await WebAssembly.instantiate(await response.arrayBuffer(), {});
        const { e4_abi_version, e4_classify } = instance.exports;
        if (e4_abi_version() !== 1) throw Error("ABI 版本不兼容");
        byId("status").textContent = "Rust WASM 已加载 · ABI 1";
        byId("analyze").disabled = false;
        byId("analyze").addEventListener("click", () => {
            try {
                const low = integer(byId("low").value), high = integer(byId("high").value);
                const raw = byId("samples").value;
                if (raw.length > 65536) throw Error("输入过长");
                const parts = raw.split(",");
                if (parts.length > 4096) throw Error("样本最多 4096 个");
                const counts = [0, 0, 0];
                for (const part of parts) {
                    const level = e4_classify(integer(part), low, high);
                    if (level < 0 || level > 2) throw Error("Rust 拒绝输入，检查阈值或完成 TODO");
                    counts[level]++;
                }
                output.textContent = "低: " + counts[0] + " | 正常: " + counts[1] + " | 高: " + counts[2];
            } catch (error) {
                output.textContent = "错误：" + error.message;
            }
        });
    } catch (error) {
        byId("status").textContent = "错误：" + error.message;
    }
})();

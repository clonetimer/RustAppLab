fn main() {
    // B4 TODO：只接受 export-json <db>，经 taskcore::load 后调用 to_json。
    // 提示 1：使用 args_os 保留操作系统路径，不调用 save。
    // 提示 2：成功只向 stdout 写 JSON 和结尾换行；错误只写 stderr。
    // 提示 3：参数非法或 load/to_json 失败时，以非零退出码结束。
    eprintln!("B4 TODO: export-json");
    std::process::exit(1);
}

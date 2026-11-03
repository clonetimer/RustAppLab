from pathlib import Path
import argparse

TEST_PY = '''"""Regression checks for the C2 handoff and non-mutating CI."""
from pathlib import Path
import unittest
ROOT = Path(__file__).resolve().parents[2]
class C2HandoffTests(unittest.TestCase):
    def test_bootstrap_generators_are_not_shipped(self):
        for name in ("scripts/c2_sync.py", ".github/workflows/c2-sync.yml", ".github/workflows/c2-bootstrap.yml", ".github/workflows/c2-green.yml"):
            self.assertFalse((ROOT/name).exists(), name)
    def test_only_main_pushes_and_prs_run_the_full_gate(self):
        text=(ROOT/".github/workflows/ci.yml").read_text(encoding="utf-8")
        self.assertTrue("branches: [main]" in text)
        self.assertFalse("branches: [main, feat/" in text)
        self.assertTrue("needs: [native, browser, msrv, c1, c2]" in text)
    def test_completion_does_not_repeat_the_old_roadmap(self):
        text=(ROOT/"apps/labquest/src/advanced_c2.rs").read_text(encoding="utf-8")
        self.assertFalse("C2 Axum 尚未交付" in text)
        self.assertTrue("stage == 1 && !cfg!(unix)" in text)
    def test_learner_fixed_assets_match_reference(self):
        base=ROOT/"projects/advanced-c2"
        for name in ("Cargo.toml", "Cargo.lock", "src/main.rs", "src/model.rs", "src/handlers.rs", "tests/contracts.rs", "assets/index.html", "assets/client.js", "assets/app.css"):
            self.assertEqual((base/"reference"/name).read_bytes(), (base/"learner"/name).read_bytes(), name)
        starter=(base/"learner/src/lib.rs").read_text(encoding="utf-8")
        self.assertEqual(starter.count("TODO("), 3)
        self.assertNotIn("persist_checked", starter)
if __name__ == "__main__": unittest.main()
'''
TEST_RS = '''
#[tokio::test]
async fn c2_1_storage_corruption_is_not_an_invalid_request() {
    let dir = Scratch::new();
    let db = dir.init();
    let app = start_store(&db).unwrap();
    app.create("saved".into()).await.unwrap();
    // A complete but semantically corrupt record: hex 20 is a blank title.
    let corrupt = b"SERVICEDESK1\\nA\\t1\\t20\\n";
    fs::write(&db, corrupt).unwrap();
    assert_eq!(app.create("valid input".into()).await, Err(ApiError::Storage));
    assert_eq!(app.list().await, Err(ApiError::Storage));
    assert_eq!(fs::read(&db).unwrap(), corrupt);
}
#[tokio::test]
async fn c2_1_failed_write_poisoning_preserves_existing_journal() {
    let dir = Scratch::new();
    let db = dir.init();
    let app = start_store(&db).unwrap();
    app.create("saved".into()).await.unwrap();
    let saved = fs::read(&db).unwrap();
    let backup = dir.0.join("backup.db");
    fs::rename(&db, &backup).unwrap();
    fs::create_dir(&db).unwrap();
    assert_eq!(app.close(1).await, Err(ApiError::Storage));
    assert_eq!(app.list().await, Err(ApiError::Storage));
    assert_eq!(fs::read(&backup).unwrap(), saved);
}
'''
HELPER_RS = '''
// Validate against the owned snapshot first. Errors from the actual persistence
// path are storage failures, even when a corrupt journal mentions a title or ID.
fn persist_checked(
    path: &Path,
    tickets: &[servicedesk::Ticket],
    action: servicedesk::Action,
) -> Result<Ticket, ApiError> {
    let mut planned = tickets.to_vec();
    let expected = servicedesk::apply(&mut planned, action.clone()).map_err(ApiError::domain)?;
    let saved = servicedesk::persist(path, action).map_err(|_| ApiError::Storage)?;
    if saved != expected { return Err(ApiError::Storage); }
    Ok(saved.into())
}

'''
def put(root, path, text):
    p=root/path; p.parent.mkdir(parents=True, exist_ok=True); p.write_bytes(text.encode('utf-8'))
def replace(root, path, old, new):
    s=(root/path).read_text(encoding='utf-8'); assert s.count(old)==1,(path,old,s.count(old)); put(root,path,s.replace(old,new))
def install_tests(root):
    put(root,'scripts/tests/test_c2_handoff.py',TEST_PY)
    for kind in ('reference','learner'):
        p=root/f'projects/advanced-c2/{kind}/tests/contracts.rs'
        assert 'c2_1_storage_corruption_is_not_an_invalid_request' not in p.read_text()
        put(root,p.relative_to(root),p.read_text(encoding='utf-8')+TEST_RS)
def apply(root):
    for path in ('.github/workflows/c2-bootstrap.yml','.github/workflows/c2-sync.yml','.github/workflows/c2-green.yml','scripts/c2_sync.py'):
        (root/path).unlink()
    replace(root,'.github/workflows/ci.yml','branches: [main, feat/c2-axum-servicedesk]','branches: [main]')
    replace(root,'.github/workflows/c2.yml','name: C2 asynchronous download qualification','name: C2 Axum service qualification')
    replace(root,'apps/labquest/src/advanced_c2.rs','C2 已通过 3/3；用自己的回环数据完成练习。C2 Axum 尚未交付。','C2 已通过 3/3；可启动自己的回环工单服务，继续验证并发与持久化行为。')
    replace(root,'apps/labquest/src/advanced_c2.rs','缺少 lib.rs/main.rs','缺少 lib.rs')
    p='projects/advanced-c2/reference/src/lib.rs';s=(root/p).read_text(encoding='utf-8')
    for action in ('Add(title)','Close(id)'):
        old=f'servicedesk::persist(&path, servicedesk::Action::{action})\n                            .map(Ticket::from)\n                            .map_err(ApiError::domain)'
        assert old in s
        s=s.replace(old,f'persist_checked(&path, &tickets, servicedesk::Action::{action})')
    old='/// Reuse the existing ServiceDesk journal and its validation rules. All synchronous\n/// disk work is done inside the dedicated blocking actor, not a Tokio I/O worker.'
    assert old in s
    s=s.replace(old,'/// Open and validate the journal synchronously during startup, before serving.\n/// Per-request disk work runs on the blocking actor, not a Tokio I/O worker.')
    anchor='/// Attach all endpoints while keeping the business core and HTML assets fixed.'; assert anchor in s
    put(root,p,s.replace(anchor,HELPER_RS+anchor))
    put(root,'projects/advanced-c2/README.md','''# C2 · Axum 异步工单服务

`reference/` 是完整参考实现，`learner/` 保留三个核心 TODO。两者独立使用 Rust 1.85.0，复用原同步 ServiceDesk 的业务规则和日志格式；基础平台的 Rust 1.74 边界不变。

参见 [C2 中文讲义](../../course/ADVANCED_C2.md)。从完整平台使用 `advanced-c2 start/prepare/check/next/lesson`；离线参考包和学员包带锁定依赖与业务库副本，可搬移后离线构建。课程验收会执行自己的可信 Rust 源码，不是恶意程序沙箱。

运行时只绑定本机回环 HTTP。启动时同步验证日志；请求期间由单个阻塞工作线程串行写盘。优雅关闭等待在途请求，不提供强制关闭截止时间；网络请求取消也不撤销已落盘的数据。持久化失败后返回存储错误并拒绝陈旧内存数据。
''')
    put(root,'course/ASYNC_NETWORK_NEXT.md','''# 异步网络课程进度与后续方向

C1 已提供 Tokio/Reqwest 回环 HTTP 下载、有限并发、超时、取消和暂存发布；C2 提供 Axum JSON 工单服务、共享状态、持久化单写者和优雅关闭。两者都有独立参考/学员工程、固定测试和离线依赖包。实际验收以对应提交的 CI 记录为准。

原 AsyncBoard 是教学执行器，NetFetch/ServiceDesk 同步课程继续保留，作为对照，不代替 Tokio 网络实现。参见 [C1 讲义](ADVANCED_C1.md)、[C2 讲义](ADVANCED_C2.md)。

后续工作聚焦公网/TLS、客户端重试与流式下载等扩展；当前 C2 不宣称生产级鉴权、断电原子性、多进程写入或强制关闭截止时间。Windows/Linux 是现阶段测试目标，macOS 仍需独立验证。
''')
    roadmap=(root/'ROADMAP.md').read_text(encoding='utf-8')
    roadmap=roadmap.replace('Tokio/Axum 真实异步网络课程 C 仍待交付。','C1 下载和 C2 Axum 工单服务已加入独立课程，具体测试结果见当前 CI。')
    roadmap=roadmap.replace('| 未交付；同步版和教学执行器不能替代 |','| C1 回环下载与 C2 Axum 工单服务：独立工程、固定验收和离线包；验证见 CI |')
    put(root,'ROADMAP.md',roadmap)
    lesson=(root/'course/ADVANCED_C2.md').read_text(encoding='utf-8')
    lesson += '\n## 故障与关闭实验补充\n\n先启动服务再让日志出现语义损坏或无法写入，合法请求应收到存储错误503，之后列表也应拒绝返回陈旧快照；非法用户输入仍返回422。损坏日志不会被自动修复。初始化日志的读取是启动阶段的同步操作，请求期间的磁盘读写才交给阻塞工作线程。\n\n优雅关闭等待已经进入的请求，不设置强制超时截止时间；遇到外部磁盘长期阻塞仍需人工处理。本机回环和请求头限制不是完整身份认证，请在可信本地环境运行。创建请求需要JSON正文，关闭请求不需要正文。\n'
    put(root,'course/ADVANCED_C2.md',lesson)
    spec=(root/'docs/superpowers/specs/2026-11-03-c2-axum-design.md').read_text(encoding='utf-8')
    spec=spec.replace('\\`','`').replace('netdesk-c2','servicedesk-c2').replace('有界地完成正在处理的请求','等待正在处理的请求完成（不设强制关闭截止时间）')
    put(root,'docs/superpowers/specs/2026-11-03-c2-axum-design.md',spec)
    plan=root/'docs/superpowers/plans/2026-11-03-c2-axum.md'
    text=plan.read_text(encoding='utf-8').replace('\\`','`')
    text+='''
## 2026-11-03 final convergence ledger
- Prior evidence: C2 Windows/Linux run41643279945 completed successfully. Final qualification must run on the newly assembled exact source tree; prior success is not substituted.
- Root cause: post-manifest README edits invalidated the inventory; c2_sync.py regenerated the grader and erased direct repairs. Removed all one-off C2 mutating workflows/generator and restored main-push + PR-only full CI. Generate the inventory once after all source changes, never trigger tests by editing tracked docs.
- Review finding: journal parsing errors mentioning a title/ID could be misclassified as invalid client input and leave the actor serving stale state. Validate client actions against the owned snapshot, map actual persistence failures to Storage, and fail closed. Added corruption and failed-write regressions; red/green evidence is stored in the final-assembly artifact.
- Ruling: retain C2's existing single-writer architecture and synchronous startup validation. Normal request disk I/O remains in spawn_blocking. Graceful shutdown has no forced deadline, now documented instead of overclaiming bounded completion. Cost: a blocked external filesystem can delay process exit.
- Ruling: template infrastructure remains fixed, while only learner implementation files determine the source digest. Contract digest includes fixed handlers, CLI, assets, legacy domain and tests. Cost: editing fixed local infrastructure is intentionally not graded.
- Final review: self-review (no subagent tool); no independent third-party review claimed. Final full CI and main checkpoint remain required before declaring the phase integrated.
'''
    put(root,plan.relative_to(root),text)
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['tests','apply']);p.add_argument('root',type=Path);a=p.parse_args()
    if a.mode=='tests': install_tests(a.root.resolve())
    else: apply(a.root.resolve())

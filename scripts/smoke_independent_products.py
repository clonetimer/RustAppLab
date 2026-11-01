#!/usr/bin/env python3
"""Maintainer-only real products demonstration. Uses fixtures, NOT learner evidence."""
import argparse
import hashlib
import json
import os
import subprocess
import tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,required=True)
    args=parser.parse_args()
    out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
    events=[]
    def run(argv, expect=0):
        env=os.environ.copy();env.pop('CARGO_BUILD_TARGET',None)
        p=subprocess.run([str(x) for x in argv],capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=120,env=env)
        n=len(events)+1
        (out/f'{n:02}-stdout.txt').write_text(p.stdout,encoding='utf-8')
        (out/f'{n:02}-stderr.txt').write_text(p.stderr,encoding='utf-8')
        events.append({'args':[str(x) for x in argv],'exit_code':p.returncode,'expected':expect})
        assert (p.returncode==0) if expect==0 else (p.returncode!=0),p.stderr
        return p
    with tempfile.TemporaryDirectory(prefix='applab-independent-') as tmpname:
        tmp=Path(tmpname)
        bins={}
        for id in ('taskpocket','pagemint'):
            project=tmp/id
            run([args.binary,'app','init',id,project])
            (project/'src/main.rs').write_bytes((ROOT/f'apps/labquest/tests/fixtures/{id}_reference.rs').read_bytes())
            # A maintainer-written regression exercises the personal-test path.
            (project/'tests/self_check.rs').write_text(own_test(id),encoding='utf-8')
            run(['cargo','test','--offline','--manifest-path',project/'Cargo.toml','--target-dir',tmp/'targets'])
            run([args.binary,'app','check',id,project])
            (out/f'{id}-APP_ASSESSMENT.md').write_bytes((project/'APP_ASSESSMENT.md').read_bytes())
            run(['cargo','build','--offline','--release','--manifest-path',project/'Cargo.toml','--target-dir',tmp/'targets'])
            bins[id]=tmp/'targets/release'/(id+('.exe' if os.name=='nt' else ''))
        db=out/'my-week.tsv';t=bins['taskpocket']
        assert run([t,'add',db,'阅读借用章节']).stdout=='added=1\n'
        assert run([t,'add',db,'写自己的测试']).stdout=='added=2\n'
        assert run([t,'done',db,'1']).stdout=='done=1\n'
        assert run([t,'stats',db]).stdout=='total=2 open=1 done=1\n'
        before=db.read_bytes();run([t,'done',db,'999'],2);assert before==db.read_bytes()
        assert run([t,'list',db]).stdout=='1\tDONE\t阅读借用章节\n2\tOPEN\t写自己的测试\n'
        notes=out/'notes';notes.mkdir()
        (notes/'03-next.note').write_text('下一步\n独立设计第三款工具。\n',encoding='utf-8')
        (notes/'01-first.note').write_text('学习计划\n记录输入与输出。\n检查真实文件。\n',encoding='utf-8')
        (notes/'02-safety.note').write_text('转义 & 文件保护\n<script> 应当是文本。\n',encoding='utf-8')
        p=bins['pagemint'];html=out/'my-notes.html'
        assert run([p,'stats',notes]).stdout=='notes=3 paragraphs=4\n'
        assert run([p,'build',notes,html]).stdout=='built=3\n'
        text=html.read_text();assert text.count('<article ')==3 and '&lt;script&gt;' in text and '<script>' not in text
        assert text.index('01-first.note')<text.index('02-safety.note')<text.index('03-next.note')
        before=html.read_bytes();run([p,'build',notes,html],2);assert before==html.read_bytes()
        bad=notes/'04-bad.note';bad.write_text('   \ninvalid\n',encoding='utf-8')
        run([p,'build',notes,out/'must-not-exist.html'],2);assert not (out/'must-not-exist.html').exists();bad.unlink()
    report={'result':'PASS','execution_count':len(events),'provenance':'maintainer reference fixtures; not human learner evidence','checks':['persistent tasks','invalid ID preserves bytes','3 notes 4 paragraphs','ordered escaped HTML','no overwrite','malformed input no artifact'],'events':events}
    (out/'RESULT.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
    (out/'README.md').write_text('# 独立作品真实运行产物\n\n由维护者基准 Rust 实现实际生成，不是学员学习效果证据。打开 my-notes.html 查看网页；my-week.tsv 是任务文件。RESULT.json 与逐条日志可复算。临时基准源码和编译缓存不包含在本包。\n',encoding='utf-8')
    (out/'SHA256SUMS.txt').write_text(''.join(f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.relative_to(out).as_posix()}\n' for p in sorted(out.rglob('*')) if p.is_file()),encoding='utf-8')
    print(json.dumps({'result':'PASS','executions':len(events),'output':str(out)},ensure_ascii=False))

def own_test(id):
    action='''let db=root.join("tasks.tsv");std::fs::write(&db,b"1\\tOPEN\\tKeep\\n").unwrap();
    let before=std::fs::read(&db).unwrap();
    let result=Command::new(env!("CARGO_BIN_EXE_taskpocket")).arg("done").arg(&db).arg("999").output().unwrap();
    assert!(!result.status.success());assert_eq!(before,std::fs::read(&db).unwrap());''' if id=='taskpocket' else '''let notes=root.join("notes");std::fs::create_dir(&notes).unwrap();std::fs::write(notes.join("one.note"),"Title\\nBody\\n").unwrap();
    let page=root.join("existing.html");std::fs::write(&page,b"existing content").unwrap();
    let result=Command::new(env!("CARGO_BIN_EXE_pagemint")).arg("build").arg(&notes).arg(&page).output().unwrap();
    assert!(!result.status.success());assert_eq!(std::fs::read(&page).unwrap(),b"existing content");'''
    return '''use std::process::Command;
#[test] fn missing_arguments_fail(){assert!(!Command::new(env!("CARGO_BIN_EXE_'''+id+'''")).output().unwrap().status.success());}
#[test] fn my_regression(){
    let root=std::env::temp_dir().join(format!("applab-own-test-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir(&root).unwrap();
    '''+action+'''
    std::fs::remove_dir_all(&root).unwrap();
}
'''
if __name__=='__main__':main()

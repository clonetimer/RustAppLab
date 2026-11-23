# RustAppLab 浏览器学习工作台

## Intent and authorization

The user asked to improve RustAppLab after comparing its split reading/editor/terminal experience with CppAppLab's integrated workbench. Deliver one usable local browser learning loop, preserving RustAppLab's existing task cards and real checks. Implementation choices below are the builder's decisions based on that request; they are not additional user requirements.

## Approach

Keep the existing Rust 1.74-compatible `labquest` binary and add a `workbench` command. Embed offline HTML/CSS/JavaScript in that binary. Serve only `127.0.0.1`; reuse existing `beginner`, `studio`, and `learn` operations. This preserves native Windows/Linux use and requires no learner Python, Node, web CDN, new Rust crate, or WSL dependency.

Alternatives considered: restyling static pages would leave tool switching intact; adding a Python/Node server would add a learner runtime dependency. A small local Rust adapter fits the current product and existing offline delivery.

## Scope and user experience

- Cover FirstCard and all 12 current Studio products through their existing data. Use the existing learning route, without inventing prerequisites or locking courses.
- A compact dark AppLab shell: course list, step navigation, tutorial/task card, source editor, and a bottom output dock. At 1366×768, actual lesson prose and source must be visible while idle. At 390/320px, single-column layout with readable steps and no page-wide horizontal overflow.
- Display each current goal, target function, concept, prediction, three progressive hints, personal-input experiment, and reflection. Preserve access to the full offline lesson.
- Create a new learner project without overwriting any existing directory. Open an existing identified learner project directly, including paths containing spaces and Chinese text; do not require newly generated documentation.
- Edit actual learner files. Save explicitly or automatically before checking/running, with a visible dirty/saving/saved/error state. A conflicting external file edit must retain the browser draft and ask the learner to reload or resolve it.
- Run the existing authoritative check for the selected step. A personal program run is a separate operation and must never create a course PASS. Show real stdout/stderr and nonzero exit status, including toolchain/environment errors.
- Idle output is compact and visible. Operations automatically reveal output. The learner may collapse/enlarge it; the collapsed state retains a meaningful result or error summary. Keep the clicked button visible and clearly pending until the operation completes.
- Preserve the learner's selection across page reload; reconstruct project/history state from the real project. Source changes invalidate any claim that an earlier successful result verifies the current source.
- Provide an obvious source/runtime Windows double-click entry and Linux command/script. Keep existing terminal workflows available. Document which courses are integrated, and direct advanced/independent course users to their established paths.

## Non-goals

No course expansion, replacement tests, fake success, automatic learner answers, cloud accounts, remote hosting, arbitrary terminal shell, or interactive stdin console. This is an interface for trusted local projects. Existing StoreDesk SQLite prerequisites and all current checker limits remain. Do not claim a new Stable release or Windows desktop user trial from CI.

## Architecture and ownership

- `workbench.rs`: CLI, loopback listener, request dispatch, application state.
- `workbench/http.rs`: bounded HTTP/form decoding and response policy.
- `workbench/model.rs`: catalog/cards, project identification/create/open, bounded regular-file access, optimistic saves, source revisions and existing navigation.
- `workbench/jobs.rs`: one active operation, existing CLI/checker subprocess execution, bounded capture, job state and source freshness.
- `workbench/ui/{index.html,app.css,app.js}`: offline, dependency-free browser interface. A plain code editor is an intentional offline baseline; do not require Monaco downloads.
- `scripts/workbench_browser_e2e.py` and native integration tests: real server/compiler checks and before/after screenshots.

### Binding platform and data constraints

Rust MSRV remains **1.74**. No new third-party crate. Windows x64 MSVC and Linux x86_64 remain supported. Host must exactly identify the loopback listener. State-changing requests require the matching Origin and a custom `X-RustAppLab: workbench` header; do not enable cross-origin access. HTTP bodies and file reads are bounded. Only explicitly selected learner projects are accessible. Reject symlinked source/file components, traversal and arbitrary file paths. Preserve raw diagnostic output as text, never executable HTML.

Project files: existing `src/**/*.rs`, `LEARNING_LOG.md` and bounded UTF-8 sample text (`samples/**` with .txt/.tsv/.csv/.json/.md). Do not allow editing Cargo.toml, tests, reports, or platform reference sources through the browser. Source revision includes execution-relevant source/manifest/sample data; learning notes do not invalidate code checks.

### HTTP contract

All JSON responses are UTF-8. Error responses are `{ "error": "human-readable Chinese message" }` with an appropriate non-2xx code. POST uses `application/x-www-form-urlencoded; charset=UTF-8` and the custom header; parse fields strictly and reject duplicate or malformed encodings. The browser must not parse terminal text into PASS.

- `GET /api/catalog` → `{version, workspace_root, courses:[{id,title,steps:[string],default_file}]}`. IDs: `firstcard`, followed by the existing 12-product learning route.
- `GET /api/course?id=ID&step=N` → `{id,title,step,step_count,step_title,goal,function,concept,prediction,hints:[string],experiment,reflection,next_course,run_args:[string],project:null|{path,files:[string],source_digest,navigation,suggested_step,history:null|{passed_steps,upto_step,same_source,compilation_failed}}}`.
- `GET /api/lesson?id=ID` → full offline lesson HTML with scripts disabled by policy.
- `POST /api/create` with `id` → current course response, step 1. A preexisting path is an error, never a deletion/reset.
- `POST /api/open` with `id,path` → current course response. Verify the selected product matches the existing project; do not regenerate it.
- `GET /api/file?id=ID&path=RELATIVE` → `{path,content,revision,source_digest}`. The file and digest come from the same snapshot; a newly loaded external edit invalidates prior current-check display.
- `POST /api/save` with `id,path,content,revision` → `{path,revision,source_digest}`. Expected revision is mandatory; mismatch returns 409 and leaves disk untouched.
- `POST /api/action` with `id,action,step,args` → job object. `action` is `check` or `run`. `args` is a newline-separated list of literal program arguments, never a shell command. Check executes `labquest learn check PROJECT STEP`; run executes real learner `cargo run --offline ... -- ARGS`. Backend validates step and argument limits before starting.
- The UI also supplies canonical `project_path` on file reads, saves and actions. The backend validates it atomically with the selected-project operation; legacy callers may omit it. Revisions and source digests include project identity. Same-course step navigation must still preserve or confirm dirty drafts if the selected project changed in another tab.
- `GET /api/job?id=NUMBER` → `{id,course_id,action,step,status,output,exit_code,source_digest,stale}`. Status is `running`, `succeeded`, or `failed`; exit code is null while running or on launch error. Polling always targets the returned job ID. `stale` prevents treating results for changed source as current.

Serve `/`, `/assets/app.css`, `/assets/app.js`, and `/api/health`. No arbitrary path serving. CLI: `labquest workbench [--workspace DIR] [--port N] [--open]`; default workspace is current directory, port 0 lets the OS choose a free loopback port. Print `RUSTAPPLAB_WORKBENCH_URL=http://127.0.0.1:PORT/` for launchers/tests. `--open` uses platform browser launch without shell interpolation; failure retains the URL and server.

## Acceptance and evidence

1. A fresh FirstCard starts incomplete, is edited and saved in the browser, passes its real checker, and changes its own output for a new name/task.
2. UnitLab starts with a real failed TODO check, passes only after a real source edit; step 2 can fail while step 1 remains a historical verified step. Actual syntax errors and their recovery are visible.
3. Existing project source is not overwritten on open. Unicode/space paths, revision conflicts, traversal/symlink rejection, duplicate busy actions, dirty navigation and disconnected-server recovery have meaningful tests.
4. All 13 course cards and stages render from existing data. Full lessons and hints are reachable. A successful ordinary run cannot be shown as formal progress.
5. Desktop and mobile before/after screenshots use the actual previous static pages and actual changed workbench, with dimensions and limitations stated. Capture pending, failure, success and compact-output states with the real backend, not fabricated success.
6. Configured native/MSRV/browser/delivery gates pass for the exact head. Verify source-manifest and packaged assets/launchers. Merge through the usual feature branch/CI workflow and verify main.

"""Run in the built image: real UIDs/Git/PG, simulated model, no API billing."""
import json
import os
from pathlib import Path
import subprocess
import shutil
import tempfile
from unittest.mock import patch

import worker as w

os.umask(0o022)
assert shutil.which("kill"), "Rust startup tests require the procps kill executable"
subprocess.run(["kill", "-0", str(os.getpid())], check=True)


def run(*args, cwd=None):
    p = subprocess.run(args, cwd=cwd, capture_output=True, text=True)
    if p.returncode:
        print(p.stderr, flush=True)
        p.check_returncode()
    return p.stdout.strip()


with tempfile.TemporaryDirectory(prefix="engineering-smoke-") as temp:
    root = Path(temp)
    root.chmod(0o755)
    seed = root / "seed"
    seed.mkdir()
    run("git", "init", "-b", "main", str(seed))
    (seed / "journal").mkdir()
    (seed / "journal" / "base.md").write_text("prior evidence\n")
    run("git", "add", ".", cwd=seed)
    run("git", "-c", "user.name=test", "-c", "user.email=test@test", "commit", "-m", "base", cwd=seed)
    run("git", "clone", "--bare", str(seed), str(root / "remote.git"))
    run("git", "clone", str(root / "remote.git"), str(root / "repository"))
    fake = root / "bin"
    fake.mkdir()
    script = fake / "codex"
    script.write_text('''#!/usr/bin/python3
import json, os, pathlib, subprocess, sys
if sys.argv[1] == 'login':
    sys.stdin.read()
else:
    assert os.getuid() == 10001
    assert not any(k in os.environ for k in ['GH_TOKEN','RAILWAY_TOKEN','CTRADER_ACCESS_TOKEN'])
    assert '127.0.0.1:15432' in os.environ['DATABASE_URL']
    try:
        pathlib.Path('/proc/' + str(os.getppid()) + '/environ').read_bytes()
    except PermissionError:
        pass
    else:
        raise AssertionError('model could read coordinator environment')
    p=subprocess.Popen(['/usr/bin/python3','-c','import time; time.sleep(300)'], start_new_session=True)
    pathlib.Path('escaped.pid').write_text(str(p.pid))
    result={'outcome':'no_change','summary':'simulated model verified UID and credential isolation',
            'tests':'real Git and local PostgreSQL checked by image smoke; no real model call',
            'owner_decisions':'none'}
    pathlib.Path(sys.argv[sys.argv.index('--output-last-message')+1]).write_text(json.dumps(result))
''')
    script.chmod(0o755)
    env = {**os.environ, "ENGINEERING_STATE_DIR": str(root), "PATH": str(fake) + ":" + os.environ["PATH"],
           "GH_TOKEN": "private-parent-token", "RAILWAY_TOKEN": "private-parent-railway",
           "OPENAI_API_KEY": "fake-model-key-not-sent-anywhere", "ENGINEERING_ISOLATION_MODE": "uid-container"}
    c = w.Coordinator(w.Config(env))
    assert c.probe_boundary(), "real external UID boundary failed"
    repo = c.prepare()
    probe_env = w.clean_environment(c.config, root, root, c.uid)
    probe = subprocess.run(["/usr/bin/python3", "-c", "import shutil,os;print(shutil.which('codex'));print(os.environ['PATH'])"],
                           env=probe_env, user=c.uid, group=c.gid, capture_output=True, text=True)
    assert probe.stdout.splitlines()[0] == str(script), "simulated CLI is not executable; refuse real model fallback"
    run("psql", "postgres://engineering@127.0.0.1:15432/postgres", "-c", "SELECT 1")
    # SQLx 0.8.6 test setup creates bookkeeping in DATABASE_URL's database
    # before it creates individual test databases (src/testing/mod.rs).
    run("psql", "postgres://engineering@127.0.0.1:15432/postgres", "-v", "ON_ERROR_STOP=1",
        "-c", "CREATE SCHEMA _sqlx_test; CREATE TABLE _sqlx_test.databases (db_name text PRIMARY KEY)")
    run("psql", "postgres://engineering@127.0.0.1:15432/postgres", "-c", "CREATE DATABASE worker_probe")
    run("psql", "postgres://engineering@127.0.0.1:15432/postgres", "-c", "DROP DATABASE worker_probe")
    denied = subprocess.run(["psql", "postgres://engineering@127.0.0.1:15432/postgres", "-v", "ON_ERROR_STOP=1",
                             "-c", "COPY (SELECT 1) TO PROGRAM 'true'"], capture_output=True)
    assert denied.returncode != 0 and b"permission denied" in denied.stderr and b"pg_execute_server_program" in denied.stderr, denied.stderr
    admin = subprocess.run(["psql", "postgres://postgres@127.0.0.1:15432/postgres", "-c", "SELECT 1"], capture_output=True)
    assert admin.returncode != 0, "model could access DB admin over TCP"
    jid = c.store.enqueue("simulated acceptance")
    job = c.store.reserve(1)
    try:
        with patch.object(c, "existing_pr", return_value=None):
            c.run_job(job, repo)
        saved = c.store.jobs()[0]
        if saved["state"] != "completed":
            for log in c.store.private.glob("*.log"):
                print(log.name + ": " + log.read_text())
            raise AssertionError(saved)
        workspace = root / "workspaces" / str(jid)
        # This command runs as supervisor/root against the model-owned worktree.
        # escaped.pid is deliberate untracked model evidence, so inspect Git directly.
        assert "escaped.pid" in c.git("ls-files", "--others", "--exclude-standard", cwd=workspace)
        pid = int((workspace / "escaped.pid").read_text())
        process = Path("/proc") / str(pid) / "stat"
        assert not process.exists() or process.read_text().split()[2] == "Z", "escaped descendant still running"
        print("PASS: actual UID isolation, root Git ownership, PostgreSQL 16, queue completion and escaped-child cleanup")
        print("PASS: real UID boundary/no-new-privileges and test DB cannot execute OS commands")
        print("SIMULATED model only: real authentication and unattended repair acceptance remain open")
    finally:
        postgres = w.pwd.getpwnam("postgres")
        subprocess.run(["pg_ctl", "-D", str(root / "postgres"), "-m", "fast", "-w", "stop"],
                       user=postgres.pw_uid, group=postgres.pw_gid, check=True, stdout=subprocess.DEVNULL)

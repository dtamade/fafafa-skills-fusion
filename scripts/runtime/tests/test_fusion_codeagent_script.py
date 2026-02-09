"""fusion-codeagent.sh 脚本测试"""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_DIR = REPO_ROOT / "scripts"


class TestFusionCodeagentScript(unittest.TestCase):
    def setUp(self):
        self.temp_dir = Path(tempfile.mkdtemp())
        self.fusion_dir = self.temp_dir / ".fusion"
        self.fusion_dir.mkdir(parents=True)
        (self.fusion_dir / "task_plan.md").write_text("### Task 1: A [PENDING]\n", encoding="utf-8")
        (self.fusion_dir / "sessions.json").write_text(
            json.dumps(
                {
                    "goal": "测试 goal",
                    "status": "in_progress",
                    "current_phase": "EXECUTE",
                    "codex_session": None,
                }
            ),
            encoding="utf-8",
        )
        (self.fusion_dir / "config.yaml").write_text(
            "backends:\n"
            "  primary: codex\n"
            "  fallback: claude\n",
            encoding="utf-8",
        )

        # mock codeagent-wrapper
        self.bin_dir = self.temp_dir / "bin"
        self.bin_dir.mkdir()
        wrapper = self.bin_dir / "codeagent-wrapper"
        wrapper.write_text(
            "#!/bin/bash\n"
            "echo \"mock backend:$2\"\n"
            "echo \"SESSION_ID: 123456\"\n",
            encoding="utf-8",
        )
        wrapper.chmod(0o755)

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_script_runs_and_updates_session_id(self):
        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        self.assertEqual(proc.returncode, 0)
        self.assertIn("mock backend", proc.stdout)

        data = json.loads((self.fusion_dir / "sessions.json").read_text(encoding="utf-8"))
        self.assertEqual(data.get("codex_session"), "123456")

    def test_fallback_updates_claude_session(self):
        # 覆盖 mock: codex 失败, claude 成功
        wrapper = self.bin_dir / "codeagent-wrapper"
        wrapper.write_text(
            "#!/bin/bash\n"
            "if [ \"$2\" = \"codex\" ]; then exit 1; fi\n"
            "echo \"mock backend:$2\"\n"
            "echo \"SESSION_ID: 654321\"\n",
            encoding="utf-8",
        )
        wrapper.chmod(0o755)

        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        self.assertEqual(proc.returncode, 0)

        data = json.loads((self.fusion_dir / "sessions.json").read_text(encoding="utf-8"))
        self.assertEqual(data.get("claude_session"), "654321")

    def test_double_backend_failure_returns_nonzero(self):
        wrapper = self.bin_dir / "codeagent-wrapper"
        wrapper.write_text("#!/bin/bash\nexit 1\n", encoding="utf-8")
        wrapper.chmod(0o755)

        env = dict(os.environ)
        env["PATH"] = f"{self.bin_dir}:{env.get('PATH', '')}"

        proc = subprocess.run(
            ["bash", str(SCRIPTS_DIR / "fusion-codeagent.sh"), "EXECUTE"],
            cwd=str(self.temp_dir),
            env=env,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        self.assertNotEqual(proc.returncode, 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)

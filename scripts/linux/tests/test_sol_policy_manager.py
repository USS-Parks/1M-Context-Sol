from __future__ import annotations

import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tempfile
import tomllib
import unittest


SCRIPT_DIR = Path(__file__).resolve().parents[1]
REPOSITORY_ROOT = SCRIPT_DIR.parents[1]
FIXTURE_PATH = (
    REPOSITORY_ROOT / "tests" / "fixtures" / "linux-policy" / "codex-0.145.0-models.json"
)
SPEC = importlib.util.spec_from_file_location(
    "sol_policy_manager", SCRIPT_DIR / "sol_policy_manager.py"
)
assert SPEC is not None and SPEC.loader is not None
manager = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(manager)


class FakeCodex:
    def __init__(
        self,
        source_catalog: bytes,
        live_output: bytes | None = None,
        version: str = "0.145.0",
    ) -> None:
        self.source_catalog = source_catalog
        self.live_output = live_output or b'{"type":"task.started","context_window":1008000}\n'
        self.version = version
        self.commands: list[list[str]] = []

    def __call__(self, command, **kwargs):
        arguments = list(command)
        self.commands.append(arguments)
        tail = arguments[1:]
        stdout = b""
        stderr = b""
        returncode = 0
        if tail == ["--version"]:
            stdout = f"codex-cli {self.version}\n".encode()
        elif tail == ["debug", "models", "--bundled"]:
            stdout = self.source_catalog
        elif tail == ["debug", "models"]:
            home = Path(kwargs["env"]["CODEX_HOME"])
            config = tomllib.loads((home / "config.toml").read_text(encoding="utf-8"))
            catalog_path = config.get("model_catalog_json")
            stdout = Path(catalog_path).read_bytes() if catalog_path else self.source_catalog
        elif tail and tail[0] == "exec":
            stdout = self.live_output
        else:
            returncode = 64
            stderr = f"unexpected fake Codex command: {arguments}".encode()
        return subprocess.CompletedProcess(arguments, returncode, stdout, stderr)


class SolPolicyManagerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="sol policy tests ")
        self.root = Path(self.temporary.name)
        self.home = self.root / "Codex Home With Spaces"
        self.home.mkdir()
        self.source = FIXTURE_PATH.read_bytes()
        self.fake = FakeCodex(self.source)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def write_config(self, data: bytes) -> None:
        (self.home / "config.toml").write_bytes(data)

    def run_action(self, *arguments: str, fake: FakeCodex | None = None):
        stdout = io.StringIO()
        stderr = io.StringIO()
        selected = fake or self.fake
        code = manager.run_cli(
            list(arguments),
            environ={"CODEX_HOME": str(self.home)},
            runner=selected,
            stdout=stdout,
            stderr=stderr,
        )
        output = json.loads(stdout.getvalue()) if stdout.getvalue() else None
        error = json.loads(stderr.getvalue()) if stderr.getvalue() else None
        return code, output, error

    def snapshot(self) -> dict[str, bytes]:
        return {
            str(path.relative_to(self.home)): path.read_bytes()
            for path in self.home.rglob("*")
            if path.is_file()
        }

    def test_plan_is_mutation_free_and_deterministic(self) -> None:
        self.write_config(b'model = "gpt-5.6-sol"\napproval_policy = "on-request"\n')
        before = self.snapshot()
        first_code, first, _ = self.run_action("plan")
        middle = self.snapshot()
        second_code, second, _ = self.run_action("plan")
        self.assertEqual((first_code, second_code), (0, 0))
        self.assertEqual(before, middle)
        self.assertEqual(before, self.snapshot())
        self.assertEqual(first, second)
        self.assertFalse(first["mutation_performed"])
        self.assertFalse(first["model_request_sent"])

    def test_fresh_install_owns_only_documented_keys_and_uses_codex_home(self) -> None:
        self.write_config(
            b'model = "gpt-5.6-sol"\napproval_policy = "never"\n\n[features]\napps = true\n'
        )
        code, result, _ = self.run_action("install")
        self.assertEqual(code, 0)
        self.assertTrue(result["installed"])
        config = tomllib.loads((self.home / "config.toml").read_text(encoding="utf-8"))
        self.assertEqual(config["model"], "gpt-5.6-sol")
        self.assertEqual(
            {key for key in config if key.startswith("model_")}, set(manager.OWNED_KEYS)
        )
        self.assertEqual(config["model_context_window"], 1_050_000)
        self.assertEqual(config["model_auto_compact_token_limit"], 900_000)
        self.assertEqual(config["model_auto_compact_token_limit_scope"], "total")
        self.assertTrue(config["features"]["apps"])
        catalog = Path(config["model_catalog_json"])
        self.assertTrue(catalog.is_absolute())
        self.assertIn("Codex Home With Spaces", str(catalog))
        manifest = json.loads(
            (self.home / "sol-1m-linux" / "state" / "install-manifest.json").read_text()
        )
        self.assertEqual(manifest["catalog"]["codex_version"], "0.145.0")
        self.assertEqual(
            manifest["catalog"]["approved_changed_fields"],
            ["auto_compact_token_limit", "context_window", "max_context_window"],
        )
        source_sol = json.loads(self.source)["models"][0]
        output_sol = json.loads(catalog.read_text())["models"][0]
        for key, value in source_sol.items():
            if key not in manager.CATALOG_POLICY_FIELDS:
                self.assertEqual(output_sol[key], value)

    def test_unknown_schema_duplicate_and_missing_sol_fail_closed(self) -> None:
        self.write_config(b'model = "gpt-5.6-sol"\n')
        source_root = json.loads(self.source)
        cases = {}
        drift = json.loads(json.dumps(source_root))
        drift["models"][0]["future_schema_field"] = True
        cases["unknown schema"] = drift
        root_drift = json.loads(json.dumps(source_root))
        root_drift["schema_version"] = 2
        cases["unknown root schema"] = root_drift
        type_drift = json.loads(json.dumps(source_root))
        type_drift["models"][0]["context_window"] = "272000"
        cases["field type drift"] = type_drift
        duplicate = json.loads(json.dumps(source_root))
        duplicate["models"].append(json.loads(json.dumps(source_root["models"][0])))
        cases["duplicate Sol"] = duplicate
        missing = json.loads(json.dumps(source_root))
        missing["models"] = []
        cases["missing Sol"] = missing
        for label, catalog in cases.items():
            with self.subTest(label=label):
                fake = FakeCodex(json.dumps(catalog).encode())
                code, _, error = self.run_action("plan", fake=fake)
                self.assertEqual(code, 2)
                self.assertTrue(error["failed_closed"])
                self.assertFalse((self.home / "sol-1m-linux").exists())

    def test_unsupported_codex_version_fails_closed(self) -> None:
        self.write_config(b'model = "gpt-5.6-sol"\n')
        fake = FakeCodex(self.source, version="0.146.0")
        code, _, error = self.run_action("plan", fake=fake)
        self.assertEqual(code, 2)
        self.assertIn("unsupported Codex version 0.146.0", error["error"])
        self.assertFalse((self.home / "sol-1m-linux").exists())

    def test_repeated_install_is_refused(self) -> None:
        self.write_config(b'model = "gpt-5.6-sol"\n')
        self.assertEqual(self.run_action("install")[0], 0)
        code, _, error = self.run_action("install")
        self.assertEqual(code, 2)
        self.assertIn("repeated install is refused", error["error"])

    def test_exact_restore_for_lf_config_with_and_without_trailing_newline(self) -> None:
        for label, original in (
            ("trailing LF", b'model = "gpt-5.6-sol"\nalpha = 1\n'),
            ("no trailing newline", b'model = "gpt-5.6-sol"\nalpha = 1'),
        ):
            with self.subTest(label=label):
                if (self.home / "sol-1m-linux").exists():
                    self.fail("prior subtest did not uninstall")
                self.write_config(original)
                self.assertEqual(self.run_action("install")[0], 0)
                code, result, _ = self.run_action("uninstall")
                self.assertEqual(code, 0)
                self.assertTrue(result["exact_restore"])
                self.assertEqual((self.home / "config.toml").read_bytes(), original)
                self.assertFalse((self.home / "sol-1m-linux").exists())

    def test_uninstall_preserves_unrelated_edits(self) -> None:
        self.write_config(
            b'model = "gpt-5.6-sol"\napproval_policy = "on-request"\n'
        )
        self.assertEqual(self.run_action("install")[0], 0)
        config_path = self.home / "config.toml"
        changed = config_path.read_bytes().replace(
            b'approval_policy = "on-request"', b'approval_policy = "never"'
        )
        config_path.write_bytes(changed)
        code, result, _ = self.run_action("uninstall")
        self.assertEqual(code, 0)
        self.assertFalse(result["exact_restore"])
        restored = config_path.read_text()
        self.assertIn('approval_policy = "never"', restored)
        self.assertNotIn(manager.BEGIN_MARKER, restored)
        for key in manager.OWNED_KEYS:
            self.assertNotIn(key, tomllib.loads(restored))

    def test_uninstall_refuses_owned_value_drift(self) -> None:
        self.write_config(b'model = "gpt-5.6-sol"\n')
        self.assertEqual(self.run_action("install")[0], 0)
        config_path = self.home / "config.toml"
        config_path.write_bytes(
            config_path.read_bytes().replace(b"model_context_window = 1050000", b"model_context_window = 777777")
        )
        code, _, error = self.run_action("uninstall")
        self.assertEqual(code, 2)
        self.assertIn("owned keys drifted", error["error"])
        self.assertTrue((self.home / "sol-1m-linux").exists())
        self.assertIn(b"777777", config_path.read_bytes())

    def test_user_owned_model_must_already_be_exact(self) -> None:
        self.write_config(b'model = "gpt-5.4"\n')
        code, _, error = self.run_action("install")
        self.assertEqual(code, 2)
        self.assertIn("user-owned top-level model", error["error"])
        self.assertEqual((self.home / "config.toml").read_bytes(), b'model = "gpt-5.4"\n')

    def test_status_reports_drift_and_never_sends_a_model_request(self) -> None:
        self.write_config(b'model = "gpt-5.6-sol"\n')
        self.assertEqual(self.run_action("install")[0], 0)
        code, result, _ = self.run_action("status")
        self.assertEqual(code, 0)
        self.assertTrue(result["catalog_compatible"])
        self.assertTrue(result["owned_values_match"])
        self.assertFalse(result["model_request_sent"])
        self.assertFalse(any(command[1:2] == ["exec"] for command in self.fake.commands))

    def test_verify_sends_live_request_only_with_explicit_flag(self) -> None:
        self.write_config(b'model = "gpt-5.6-sol"\n')
        self.assertEqual(self.run_action("install")[0], 0)
        code, result, _ = self.run_action("verify")
        self.assertEqual(code, 0)
        self.assertFalse(result["live_model_request_sent"])
        self.assertFalse(any(command[1:2] == ["exec"] for command in self.fake.commands))
        code, result, _ = self.run_action("verify", "--live")
        self.assertEqual(code, 0)
        self.assertTrue(result["live_model_request_sent"])
        self.assertEqual(result["verified_host_budget"], 1_008_000)
        code, status_result, _ = self.run_action("status")
        self.assertEqual(code, 0)
        self.assertEqual(
            status_result["latest_verified_host_budget"]["tokens"], 1_008_000
        )


if __name__ == "__main__":
    unittest.main()

#!/usr/bin/env python3
"""Fail-closed Linux installer for the GPT-5.6 Sol 1M Codex catalog policy."""

from __future__ import annotations

import argparse
import base64
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import tomllib
from typing import Any, Callable, Mapping, Sequence


MANAGER_VERSION = "1.0.0"
MANIFEST_SCHEMA_VERSION = 1
MODEL = "gpt-5.6-sol"
CONTEXT_WINDOW = 1_050_000
AUTO_COMPACT_LIMIT = 900_000
AUTO_COMPACT_SCOPE = "total"
INSTALL_DIRECTORY = "sol-1m-linux"
CATALOG_FILE = "generated-sol-1m-models.json"
MANIFEST_FILE = "install-manifest.json"
BACKUP_FILE = "config.before.toml"
VERIFICATION_FILE = "latest-verification.json"
BEGIN_MARKER = "# BEGIN 1M Context Sol Linux managed policy"
END_MARKER = "# END 1M Context Sol Linux managed policy"

OWNED_KEYS = (
    "model_context_window",
    "model_auto_compact_token_limit",
    "model_auto_compact_token_limit_scope",
    "model_catalog_json",
)
CATALOG_POLICY_FIELDS = (
    "auto_compact_token_limit",
    "context_window",
    "max_context_window",
)

# Exact Sol entry keys in the official Codex 0.145.0 release catalog. A field
# addition/removal is treated as schema drift and requires a reviewed profile.
CODEX_0145_SOL_FIELDS = frozenset(
    {
        "additional_speed_tiers",
        "apply_patch_tool_type",
        "auto_compact_token_limit",
        "auto_review_model_override",
        "availability_nux",
        "available_in_plans",
        "base_instructions",
        "comp_hash",
        "context_window",
        "default_reasoning_level",
        "default_reasoning_summary",
        "default_service_tier",
        "default_verbosity",
        "description",
        "display_name",
        "experimental_supported_tools",
        "include_skills_usage_instructions",
        "input_modalities",
        "max_context_window",
        "minimal_client_version",
        "model_messages",
        "multi_agent_version",
        "prefer_websockets",
        "priority",
        "reasoning_summary_format",
        "service_tiers",
        "shell_type",
        "slug",
        "support_verbosity",
        "supported_in_api",
        "supported_reasoning_levels",
        "supports_image_detail_original",
        "supports_parallel_tool_calls",
        "supports_reasoning_summaries",
        "supports_search_tool",
        "tool_mode",
        "truncation_policy",
        "upgrade",
        "use_responses_lite",
        "visibility",
        "web_search_tool_type",
    }
)
SUPPORTED_PROFILES = {
    "0.145.0": {
        "schema_id": "codex-model-catalog/0.145.0-v1",
        "sol_fields": CODEX_0145_SOL_FIELDS,
    }
}


class ManagerError(RuntimeError):
    """Expected fail-closed policy error."""


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def deterministic_json(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    ).encode("utf-8")


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")


def is_integer(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def decode_utf8(data: bytes, label: str) -> str:
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ManagerError(f"{label} is not valid UTF-8: {error}") from error


def parse_toml(data: bytes, label: str) -> dict[str, Any]:
    text = decode_utf8(data, label)
    try:
        parsed = tomllib.loads(text)
    except tomllib.TOMLDecodeError as error:
        raise ManagerError(f"{label} is not valid TOML: {error}") from error
    if not isinstance(parsed, dict):
        raise ManagerError(f"{label} did not parse as a TOML document")
    return parsed


def require_regular_file(path: Path, label: str) -> bytes:
    try:
        info = path.lstat()
    except FileNotFoundError as error:
        raise ManagerError(f"{label} does not exist: {path}") from error
    if stat.S_ISLNK(info.st_mode) or not stat.S_ISREG(info.st_mode):
        raise ManagerError(f"{label} must be a regular non-symlink file: {path}")
    return path.read_bytes()


def require_codex_home(environ: Mapping[str, str]) -> Path:
    configured = environ.get("CODEX_HOME")
    home = Path(configured).expanduser() if configured else Path.home() / ".codex"
    if not home.is_absolute():
        raise ManagerError("CODEX_HOME must be an absolute path")
    try:
        info = home.lstat()
    except FileNotFoundError as error:
        raise ManagerError(f"CODEX_HOME does not exist: {home}") from error
    if stat.S_ISLNK(info.st_mode) or not stat.S_ISDIR(info.st_mode):
        raise ManagerError(f"CODEX_HOME must be a non-symlink directory: {home}")
    return home


def normalize_codex_version(output: bytes) -> str:
    text = decode_utf8(output, "Codex version output").strip()
    match = re.fullmatch(r"codex-cli\s+(\d+\.\d+\.\d+)", text)
    if not match:
        raise ManagerError(f"unrecognized Codex version output: {text!r}")
    version = match.group(1)
    if version not in SUPPORTED_PROFILES:
        supported = ", ".join(sorted(SUPPORTED_PROFILES))
        raise ManagerError(
            f"unsupported Codex version {version}; supported Linux profiles: {supported}"
        )
    return version


Runner = Callable[..., subprocess.CompletedProcess[bytes]]


class CodexClient:
    def __init__(
        self,
        command: str,
        codex_home: Path,
        runner: Runner = subprocess.run,
        environ: Mapping[str, str] | None = None,
    ) -> None:
        self.command = command
        self.codex_home = codex_home
        self.runner = runner
        self.environ = dict(os.environ if environ is None else environ)
        self.environ["CODEX_HOME"] = str(codex_home)

    def run(self, arguments: Sequence[str], timeout: int = 60) -> bytes:
        command = [self.command, *arguments]
        try:
            result = self.runner(
                command,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
                timeout=timeout,
                env=self.environ,
            )
        except (OSError, subprocess.SubprocessError) as error:
            raise ManagerError(f"could not run {' '.join(command)}: {error}") from error
        if result.returncode != 0:
            detail = decode_utf8(result.stderr or b"", "Codex stderr").strip()
            suffix = f": {detail}" if detail else ""
            raise ManagerError(
                f"Codex command failed ({result.returncode}): {' '.join(command)}{suffix}"
            )
        return result.stdout or b""

    def version(self) -> str:
        return normalize_codex_version(self.run(["--version"]))

    def source_catalog(self) -> tuple[str, bytes]:
        version = self.version()
        # Bundled output is version-matched and avoids network/model traffic or
        # recursively reading an already-installed model_catalog_json.
        return version, self.run(["debug", "models", "--bundled"])

    def resolved_catalog(self) -> bytes:
        return self.run(["debug", "models"])

    def live_probe(self, timeout: int) -> bytes:
        return self.run(
            [
                "exec",
                "--json",
                "--ephemeral",
                "--skip-git-repo-check",
                "--sandbox",
                "read-only",
                "--model",
                MODEL,
                "-c",
                'approval_policy="never"',
                "Return exactly SOL_POLICY_LIVE_OK. Do not use tools.",
            ],
            timeout=timeout,
        )


def validate_sol_types(sol: dict[str, Any]) -> None:
    string_fields = {
        "apply_patch_tool_type",
        "base_instructions",
        "comp_hash",
        "default_reasoning_level",
        "default_reasoning_summary",
        "display_name",
        "minimal_client_version",
        "reasoning_summary_format",
        "shell_type",
        "slug",
        "tool_mode",
        "visibility",
        "web_search_tool_type",
    }
    string_or_null_fields = {
        "default_service_tier",
        "default_verbosity",
        "description",
        "multi_agent_version",
    }
    boolean_fields = {
        "include_skills_usage_instructions",
        "prefer_websockets",
        "support_verbosity",
        "supported_in_api",
        "supports_image_detail_original",
        "supports_parallel_tool_calls",
        "supports_reasoning_summaries",
        "supports_search_tool",
        "use_responses_lite",
    }
    array_fields = {
        "additional_speed_tiers",
        "available_in_plans",
        "experimental_supported_tools",
        "input_modalities",
        "service_tiers",
        "supported_reasoning_levels",
    }
    object_or_null_fields = {
        "auto_review_model_override",
        "availability_nux",
        "model_messages",
        "upgrade",
    }
    for field in string_fields:
        if not isinstance(sol[field], str):
            raise ManagerError(f"Sol catalog field {field!r} must be a string")
    for field in string_or_null_fields:
        if sol[field] is not None and not isinstance(sol[field], str):
            raise ManagerError(f"Sol catalog field {field!r} must be a string or null")
    for field in boolean_fields:
        if not isinstance(sol[field], bool):
            raise ManagerError(f"Sol catalog field {field!r} must be a boolean")
    for field in array_fields:
        if not isinstance(sol[field], list):
            raise ManagerError(f"Sol catalog field {field!r} must be an array")
    for field in object_or_null_fields:
        if sol[field] is not None and not isinstance(sol[field], dict):
            raise ManagerError(f"Sol catalog field {field!r} must be an object or null")
    for field in ("context_window", "max_context_window", "priority"):
        if not is_integer(sol[field]):
            raise ManagerError(f"Sol catalog field {field!r} must be an integer")
    compact = sol["auto_compact_token_limit"]
    if compact is not None and not is_integer(compact):
        raise ManagerError(
            "Sol catalog field 'auto_compact_token_limit' must be an integer or null"
        )
    if not sol["base_instructions"]:
        raise ManagerError("Sol catalog base_instructions must not be empty")


def validate_catalog(raw: bytes, version: str) -> tuple[dict[str, Any], dict[str, Any]]:
    try:
        root = json.loads(decode_utf8(raw, "Codex model catalog"))
    except json.JSONDecodeError as error:
        raise ManagerError(f"Codex model catalog is invalid JSON: {error}") from error
    if not isinstance(root, dict) or set(root) != {"models"}:
        raise ManagerError("unknown catalog root schema; expected only 'models'")
    models = root["models"]
    if not isinstance(models, list):
        raise ManagerError("catalog 'models' must be an array")
    slugs: set[str] = set()
    matching: list[dict[str, Any]] = []
    for entry in models:
        if not isinstance(entry, dict) or not isinstance(entry.get("slug"), str):
            raise ManagerError("every catalog model must be an object with a string slug")
        slug = entry["slug"]
        if slug in slugs:
            raise ManagerError(f"catalog contains duplicate model slug {slug!r}")
        slugs.add(slug)
        if slug == MODEL:
            matching.append(entry)
    if len(matching) != 1:
        raise ManagerError(
            f"catalog must contain exactly one {MODEL!r} entry; found {len(matching)}"
        )
    sol = matching[0]
    expected = SUPPORTED_PROFILES[version]["sol_fields"]
    actual = frozenset(sol)
    if actual != expected:
        missing = sorted(expected - actual)
        unknown = sorted(actual - expected)
        raise ManagerError(
            f"unknown Sol schema drift for Codex {version}; missing={missing}, unknown={unknown}"
        )
    validate_sol_types(sol)
    return root, sol


def generate_catalog(raw: bytes, version: str) -> tuple[bytes, dict[str, Any]]:
    root, source_sol = validate_catalog(raw, version)
    output_sol = json.loads(json.dumps(source_sol))
    output_sol["context_window"] = CONTEXT_WINDOW
    output_sol["max_context_window"] = CONTEXT_WINDOW
    output_sol["auto_compact_token_limit"] = AUTO_COMPACT_LIMIT

    preserved_source = {
        key: value for key, value in source_sol.items() if key not in CATALOG_POLICY_FIELDS
    }
    preserved_output = {
        key: value for key, value in output_sol.items() if key not in CATALOG_POLICY_FIELDS
    }
    if preserved_source != preserved_output:
        raise ManagerError("catalog generation changed a field outside the policy allowlist")
    actual_changes = sorted(
        field
        for field in CATALOG_POLICY_FIELDS
        if source_sol.get(field) != output_sol.get(field)
    )
    if not actual_changes:
        raise ManagerError("source catalog already contains the complete requested policy")

    output_root = {"models": [output_sol]}
    output = deterministic_json(output_root)
    generation = {
        "schema_id": SUPPORTED_PROFILES[version]["schema_id"],
        "codex_version": version,
        "model": MODEL,
        "source_catalog_sha256": sha256(raw),
        "source_normalized_sha256": sha256(canonical_json(root)),
        "source_sol_sha256": sha256(canonical_json(source_sol)),
        "preserved_sol_sha256": sha256(canonical_json(preserved_source)),
        "output_catalog_sha256": sha256(output),
        "output_sol_sha256": sha256(canonical_json(output_sol)),
        "output_model_count": 1,
        "approved_changed_fields": list(CATALOG_POLICY_FIELDS),
        "actual_changed_fields": actual_changes,
        "policy": {
            "context_window": CONTEXT_WINDOW,
            "max_context_window": CONTEXT_WINDOW,
            "auto_compact_token_limit": AUTO_COMPACT_LIMIT,
        },
    }
    return output, generation


def managed_values(catalog_path: Path) -> dict[str, Any]:
    return {
        "model_context_window": CONTEXT_WINDOW,
        "model_auto_compact_token_limit": AUTO_COMPACT_LIMIT,
        "model_auto_compact_token_limit_scope": AUTO_COMPACT_SCOPE,
        "model_catalog_json": str(catalog_path),
    }


def managed_block(catalog_path: Path) -> bytes:
    values = managed_values(catalog_path)
    lines = [
        BEGIN_MARKER,
        f"model_context_window = {values['model_context_window']}",
        f"model_auto_compact_token_limit = {values['model_auto_compact_token_limit']}",
        "model_auto_compact_token_limit_scope = "
        + json.dumps(values["model_auto_compact_token_limit_scope"]),
        "model_catalog_json = " + json.dumps(values["model_catalog_json"]),
        END_MARKER,
        "",
    ]
    return "\n".join(lines).encode("utf-8")


def validate_original_config(data: bytes) -> dict[str, Any]:
    config = parse_toml(data, "Codex config")
    if config.get("model") != MODEL:
        raise ManagerError(
            f"user-owned top-level model must already be exact {MODEL!r}; it is never installed or owned"
        )
    conflicts = [key for key in OWNED_KEYS if key in config]
    if conflicts:
        raise ManagerError(f"owned-key conflict in Codex config: {conflicts}")
    return config


def append_managed_config(original: bytes, catalog_path: Path) -> tuple[bytes, bytes]:
    validate_original_config(original)
    block = managed_block(catalog_path)
    # Prepending keeps every managed assignment in TOML's top-level scope even
    # when the user's file ends inside a table such as [features]. The original
    # bytes remain one untouched suffix for exact restoration.
    candidate = block + original
    parsed = parse_toml(candidate, "candidate Codex config")
    expected = managed_values(catalog_path)
    for key, value in expected.items():
        if parsed.get(key) != value:
            raise ManagerError(f"candidate config did not resolve owned key {key!r} exactly")
    return candidate, block


def paths_for(codex_home: Path) -> dict[str, Path]:
    install_root = codex_home / INSTALL_DIRECTORY
    state = install_root / "state"
    return {
        "codex_home": codex_home,
        "config": codex_home / "config.toml",
        "install_root": install_root,
        "state": state,
        "catalog": install_root / CATALOG_FILE,
        "manifest": state / MANIFEST_FILE,
        "backup": state / BACKUP_FILE,
        "verification": state / VERIFICATION_FILE,
    }


def build_candidate(
    paths: Mapping[str, Path], client: CodexClient
) -> dict[str, Any]:
    original = require_regular_file(paths["config"], "Codex config")
    version, source = client.source_catalog()
    catalog, catalog_manifest = generate_catalog(source, version)
    candidate_config, block = append_managed_config(original, paths["catalog"])
    manifest = {
        "schema_version": MANIFEST_SCHEMA_VERSION,
        "manager_version": MANAGER_VERSION,
        "platform": "linux-headless",
        "model": MODEL,
        "codex_home": str(paths["codex_home"]),
        "files": {
            "config": str(paths["config"]),
            "catalog": str(paths["catalog"]),
            "backup": str(paths["backup"]),
            "manifest": str(paths["manifest"]),
            "verification": str(paths["verification"]),
        },
        "config": {
            "original_sha256": sha256(original),
            "installed_sha256": sha256(candidate_config),
            "original_ended_with_newline": original.endswith((b"\n", b"\r")),
            "owned_keys": list(OWNED_KEYS),
            "owned_values": managed_values(paths["catalog"]),
            "managed_block_sha256": sha256(block),
            "managed_block_base64": base64.b64encode(block).decode("ascii"),
            "user_owned_model": MODEL,
        },
        "catalog": catalog_manifest,
        "process_control": {
            "codex_restart_required": True,
            "paseo_daemon_restart_automatic": False,
        },
    }
    manifest_bytes = deterministic_json(manifest)
    return {
        "original": original,
        "candidate_config": candidate_config,
        "catalog": catalog,
        "manifest": manifest,
        "manifest_bytes": manifest_bytes,
    }


def atomic_write(path: Path, data: bytes, mode_from: Path | None = None) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        if mode_from is not None:
            os.chmod(temporary, stat.S_IMODE(mode_from.stat().st_mode))
        os.replace(temporary, path)
    finally:
        if temporary.exists():
            temporary.unlink()


def read_manifest(paths: Mapping[str, Path]) -> dict[str, Any]:
    raw = require_regular_file(paths["manifest"], "Linux policy manifest")
    try:
        manifest = json.loads(decode_utf8(raw, "Linux policy manifest"))
    except json.JSONDecodeError as error:
        raise ManagerError(f"Linux policy manifest is invalid JSON: {error}") from error
    if not isinstance(manifest, dict):
        raise ManagerError("Linux policy manifest root must be an object")
    if manifest.get("schema_version") != MANIFEST_SCHEMA_VERSION:
        raise ManagerError("unsupported Linux policy manifest schema")
    if manifest.get("manager_version") != MANAGER_VERSION:
        raise ManagerError("installed Linux policy manager version is unsupported")
    if manifest.get("codex_home") != str(paths["codex_home"]):
        raise ManagerError("manifest CODEX_HOME does not match the active CODEX_HOME")
    expected_files = {
        "config": str(paths["config"]),
        "catalog": str(paths["catalog"]),
        "backup": str(paths["backup"]),
        "manifest": str(paths["manifest"]),
        "verification": str(paths["verification"]),
    }
    if manifest.get("files") != expected_files:
        raise ManagerError("manifest file ownership paths do not match this installation")
    return manifest


def inspect_owned_values(config_data: bytes, manifest: Mapping[str, Any]) -> dict[str, Any]:
    parsed = parse_toml(config_data, "installed Codex config")
    expected = manifest["config"]["owned_values"]
    drift = {
        key: {"expected": expected[key], "actual": parsed.get(key)}
        for key in OWNED_KEYS
        if parsed.get(key) != expected[key]
    }
    return {
        "match": not drift,
        "drift": drift,
        "user_model_exact": parsed.get("model") == MODEL,
    }


def inspect_catalog_compatibility(
    paths: Mapping[str, Path], manifest: Mapping[str, Any], client: CodexClient
) -> dict[str, Any]:
    try:
        version, source = client.source_catalog()
        _, source_sol = validate_catalog(source, version)
        catalog_data = require_regular_file(paths["catalog"], "generated Sol catalog")
        if sha256(catalog_data) != manifest["catalog"]["output_catalog_sha256"]:
            raise ManagerError("generated Sol catalog hash drifted from the manifest")
        _, installed_sol = validate_catalog(catalog_data, version)
        resolved_data = client.resolved_catalog()
        _, resolved_sol = validate_catalog(resolved_data, version)
        if sha256(source) != manifest["catalog"]["source_catalog_sha256"]:
            raise ManagerError("installed Codex bundled source catalog changed")
        if installed_sol != resolved_sol:
            raise ManagerError("codex debug models did not resolve the installed catalog exactly")
        expected_policy = {
            "context_window": CONTEXT_WINDOW,
            "max_context_window": CONTEXT_WINDOW,
            "auto_compact_token_limit": AUTO_COMPACT_LIMIT,
        }
        if any(installed_sol[key] != value for key, value in expected_policy.items()):
            raise ManagerError("resolved Sol catalog policy values do not match")
        preserved_source = {
            key: value for key, value in source_sol.items() if key not in CATALOG_POLICY_FIELDS
        }
        preserved_installed = {
            key: value
            for key, value in installed_sol.items()
            if key not in CATALOG_POLICY_FIELDS
        }
        if preserved_source != preserved_installed:
            raise ManagerError("installed Sol catalog drifted outside the policy allowlist")
        return {"compatible": True, "error": None, "codex_version": version}
    except ManagerError as error:
        return {"compatible": False, "error": str(error), "codex_version": None}


def latest_verified_budget(paths: Mapping[str, Path]) -> dict[str, Any] | None:
    if not paths["verification"].exists():
        return None
    try:
        record = json.loads(paths["verification"].read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError):
        return None
    budget = record.get("verified_host_budget")
    if not is_integer(budget):
        return None
    return {
        "tokens": budget,
        "source_field": record.get("verified_host_budget_source_field"),
        "verified_at_utc": record.get("verified_at_utc"),
    }


def plan(paths: Mapping[str, Path], client: CodexClient) -> dict[str, Any]:
    if paths["install_root"].exists():
        raise ManagerError(f"install root already exists: {paths['install_root']}")
    candidate = build_candidate(paths, client)
    return {
        "action": "plan",
        "mutation_performed": False,
        "codex_version": candidate["manifest"]["catalog"]["codex_version"],
        "schema_id": candidate["manifest"]["catalog"]["schema_id"],
        "files": candidate["manifest"]["files"],
        "owned_keys": list(OWNED_KEYS),
        "owned_values": candidate["manifest"]["config"]["owned_values"],
        "user_owned_model_required": MODEL,
        "source_catalog_sha256": candidate["manifest"]["catalog"][
            "source_catalog_sha256"
        ],
        "candidate_catalog_sha256": sha256(candidate["catalog"]),
        "candidate_config_sha256": sha256(candidate["candidate_config"]),
        "candidate_manifest_sha256": sha256(candidate["manifest_bytes"]),
        "codex_restart_required": True,
        "paseo_daemon_restart_automatic": False,
        "model_request_sent": False,
    }


def install(paths: Mapping[str, Path], client: CodexClient) -> dict[str, Any]:
    if paths["install_root"].exists():
        raise ManagerError(
            f"installation already exists; repeated install is refused: {paths['install_root']}"
        )
    candidate = build_candidate(paths, client)
    paths["install_root"].mkdir(mode=0o700, parents=False, exist_ok=False)
    config_changed = False
    try:
        paths["state"].mkdir(mode=0o700)
        atomic_write(paths["backup"], candidate["original"])
        atomic_write(paths["catalog"], candidate["catalog"])
        atomic_write(paths["manifest"], candidate["manifest_bytes"])
        atomic_write(
            paths["config"], candidate["candidate_config"], mode_from=paths["config"]
        )
        config_changed = True
    except Exception:
        if config_changed:
            atomic_write(paths["config"], candidate["original"], mode_from=paths["config"])
        shutil.rmtree(paths["install_root"], ignore_errors=True)
        raise
    return {
        "action": "install",
        "installed": True,
        "manager_version": MANAGER_VERSION,
        "codex_version": candidate["manifest"]["catalog"]["codex_version"],
        "catalog_path": str(paths["catalog"]),
        "manifest_path": str(paths["manifest"]),
        "owned_keys": list(OWNED_KEYS),
        "user_owned_model": MODEL,
        "codex_restart_required": True,
        "paseo_daemon_restarted": False,
        "model_request_sent": False,
    }


def status(paths: Mapping[str, Path], client: CodexClient) -> dict[str, Any]:
    if not paths["manifest"].exists():
        try:
            detected = client.version()
            version_error = None
        except ManagerError as error:
            detected = None
            version_error = str(error)
        return {
            "action": "status",
            "installed": False,
            "manager_version": None,
            "detected_codex_version": detected,
            "codex_version_error": version_error,
            "owned_values_match": None,
            "catalog_compatible": None,
            "latest_verified_host_budget": None,
            "model_request_sent": False,
        }
    manifest = read_manifest(paths)
    config_data = require_regular_file(paths["config"], "Codex config")
    owned = inspect_owned_values(config_data, manifest)
    compatible = inspect_catalog_compatibility(paths, manifest, client)
    return {
        "action": "status",
        "installed": True,
        "manager_version": manifest["manager_version"],
        "installed_codex_version": manifest["catalog"]["codex_version"],
        "detected_codex_version": compatible["codex_version"],
        "owned_values_match": owned["match"],
        "owned_value_drift": owned["drift"],
        "user_owned_model_exact": owned["user_model_exact"],
        "config_snapshot_matches": sha256(config_data)
        == manifest["config"]["installed_sha256"],
        "catalog_compatible": compatible["compatible"],
        "catalog_compatibility_error": compatible["error"],
        "latest_verified_host_budget": latest_verified_budget(paths),
        "codex_restart_required_after_config_change": True,
        "paseo_daemon_restart_automatic": False,
        "model_request_sent": False,
    }


def remove_managed_block(
    current: bytes, original: bytes, manifest: Mapping[str, Any]
) -> bytes:
    owned = inspect_owned_values(current, manifest)
    if not owned["match"]:
        raise ManagerError(f"refusing uninstall because owned keys drifted: {owned['drift']}")
    try:
        block = base64.b64decode(
            manifest["config"]["managed_block_base64"], validate=True
        )
    except (KeyError, ValueError) as error:
        raise ManagerError("manifest managed block is invalid") from error
    if sha256(block) != manifest["config"]["managed_block_sha256"]:
        raise ManagerError("manifest managed block hash does not match")
    if current.count(block) != 1:
        raise ManagerError("refusing uninstall because the exact managed block changed")
    start = current.index(block)
    before = current[:start]
    after = current[start + len(block) :]
    candidate = before + after
    parsed = parse_toml(candidate, "Codex config after managed-key removal")
    if parsed.get("model") != MODEL:
        raise ManagerError("refusing uninstall because the user-owned model is no longer exact")
    if any(key in parsed for key in OWNED_KEYS):
        raise ManagerError("refusing uninstall because an owned key remains after block removal")
    if candidate == current:
        raise ManagerError("managed-key removal made no change")
    if current == original:
        raise ManagerError("internal uninstall state is inconsistent")
    return candidate


def uninstall(paths: Mapping[str, Path]) -> dict[str, Any]:
    manifest = read_manifest(paths)
    current = require_regular_file(paths["config"], "Codex config")
    original = require_regular_file(paths["backup"], "original config backup")
    if sha256(original) != manifest["config"]["original_sha256"]:
        raise ManagerError("original config backup hash does not match the manifest")
    if sha256(current) == manifest["config"]["installed_sha256"]:
        restored = original
        exact = True
    else:
        restored = remove_managed_block(current, original, manifest)
        exact = restored == original
    atomic_write(paths["config"], restored, mode_from=paths["config"])
    shutil.rmtree(paths["install_root"])
    return {
        "action": "uninstall",
        "removed": True,
        "exact_restore": exact,
        "preserved_later_unrelated_edits": not exact,
        "config_sha256": sha256(restored),
        "codex_restart_required": True,
        "paseo_daemon_restarted": False,
        "model_request_sent": False,
    }


def walk_budget(value: Any) -> tuple[int | None, str | None]:
    priority = ("effective_task_budget", "task_budget", "context_window")
    if isinstance(value, dict):
        for key in priority:
            if is_integer(value.get(key)) and value[key] > 0:
                return value[key], key
        for child in value.values():
            budget, field = walk_budget(child)
            if budget is not None:
                return budget, field
    elif isinstance(value, list):
        for child in value:
            budget, field = walk_budget(child)
            if budget is not None:
                return budget, field
    return None, None


def extract_live_budget(output: bytes) -> tuple[int | None, str | None]:
    for line in output.splitlines():
        try:
            event = json.loads(decode_utf8(line, "Codex live JSONL event"))
        except json.JSONDecodeError:
            continue
        budget, field = walk_budget(event)
        if budget is not None:
            return budget, field
    return None, None


def verify(
    paths: Mapping[str, Path], client: CodexClient, live: bool, timeout: int
) -> dict[str, Any]:
    report = status(paths, client)
    if not report["installed"]:
        raise ManagerError("Linux Sol policy is not installed")
    if not report["owned_values_match"] or not report["user_owned_model_exact"]:
        raise ManagerError("Codex config ownership or user model drift prevents verification")
    if not report["catalog_compatible"]:
        raise ManagerError(
            f"catalog verification failed: {report['catalog_compatibility_error']}"
        )
    budget = None
    budget_field = None
    if live:
        output = client.live_probe(timeout)
        budget, budget_field = extract_live_budget(output)
    record = {
        "schema_version": 1,
        "verified_at_utc": dt.datetime.now(dt.timezone.utc)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z"),
        "codex_version": report["detected_codex_version"],
        "model": MODEL,
        "catalog_verified": True,
        "catalog_context_window": CONTEXT_WINDOW,
        "catalog_auto_compact_token_limit": AUTO_COMPACT_LIMIT,
        "live_model_request_sent": live,
        "verified_host_budget": budget,
        "verified_host_budget_source_field": budget_field,
    }
    atomic_write(paths["verification"], deterministic_json(record))
    return {
        "action": "verify",
        "verified": True,
        "catalog_verified": True,
        "live_model_request_sent": live,
        "verified_host_budget": budget,
        "verified_host_budget_source_field": budget_field,
        "verification_record": str(paths["verification"]),
        "paseo_daemon_restarted": False,
    }


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Manage the headless Linux GPT-5.6 Sol 1M Codex policy"
    )
    parser.add_argument("action", choices=("plan", "install", "status", "verify", "uninstall"))
    parser.add_argument(
        "--codex", default="codex", help="Codex executable name or absolute path"
    )
    parser.add_argument(
        "--live",
        action="store_true",
        help="verify only: explicitly permit one fresh gpt-5.6-sol model request",
    )
    parser.add_argument(
        "--live-timeout-seconds", type=int, default=300, help=argparse.SUPPRESS
    )
    return parser


def run_cli(
    argv: Sequence[str] | None = None,
    *,
    environ: Mapping[str, str] | None = None,
    runner: Runner = subprocess.run,
    stdout: Any = sys.stdout,
    stderr: Any = sys.stderr,
) -> int:
    args = build_parser().parse_args(argv)
    environment = dict(os.environ if environ is None else environ)
    try:
        codex_home = require_codex_home(environment)
        paths = paths_for(codex_home)
        client = CodexClient(args.codex, codex_home, runner=runner, environ=environment)
        if args.live and args.action != "verify":
            raise ManagerError("--live is accepted only with verify")
        if args.live_timeout_seconds <= 0:
            raise ManagerError("--live-timeout-seconds must be positive")
        if args.action == "plan":
            result = plan(paths, client)
        elif args.action == "install":
            result = install(paths, client)
        elif args.action == "status":
            result = status(paths, client)
        elif args.action == "verify":
            result = verify(paths, client, args.live, args.live_timeout_seconds)
        else:
            result = uninstall(paths)
        stdout.write(deterministic_json(result).decode("utf-8"))
        return 0
    except (ManagerError, OSError, KeyError, TypeError) as error:
        payload = {"error": str(error), "failed_closed": True}
        stderr.write(deterministic_json(payload).decode("utf-8"))
        return 2


if __name__ == "__main__":
    raise SystemExit(run_cli())

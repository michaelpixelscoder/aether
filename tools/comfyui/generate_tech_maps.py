#!/usr/bin/env python3
"""Generate reviewed technical maps for one convention-based asset directory."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path


TOOL_DIR = Path(__file__).resolve().parent
PROJECT_DIR = TOOL_DIR.parents[1]
SERVER = "http://127.0.0.1:8188"


def server_ready() -> bool:
    try:
        with urllib.request.urlopen(f"{SERVER}/system_stats", timeout=2):
            return True
    except (urllib.error.URLError, TimeoutError):
        return False


def resolve(asset_dir: Path, value: str) -> Path:
    return (asset_dir / value).resolve()


def main() -> None:
    parser = argparse.ArgumentParser(
        usage="generate_tech_maps asset_directory",
        description="Generate normal/AO/ORM maps with the asset's pinned ComfyUI workflow.",
    )
    parser.add_argument("asset_directory", type=Path)
    args = parser.parse_args()
    asset_dir = args.asset_directory.resolve()
    if not asset_dir.is_dir():
        raise SystemExit(f"asset directory does not exist: {asset_dir}")

    config_path = asset_dir / "tech_maps.json"
    config = json.loads(config_path.read_text()) if config_path.is_file() else {}
    prepare = resolve(asset_dir, config.get("prepare", "source/prepare_inputs.py"))
    workflow = resolve(asset_dir, config.get("workflow", "source/workflow.api.json"))
    input_dir = resolve(asset_dir, config.get("input_dir", "source/inputs"))
    output_dir = resolve(asset_dir, config.get("output_dir", "textures"))
    validator = resolve(asset_dir, config.get("validator", "validate.py"))
    prompt = asset_dir / "texture_prompt.md"
    namespace = config.get("input_namespace", f"aether/{asset_dir.name}")
    authoring_python = TOOL_DIR / ".runtime" / "venv" / "bin" / "python"
    if not authoring_python.is_file():
        raise SystemExit(f"ComfyUI is not installed; run {TOOL_DIR / 'setup.sh'} first")

    for required in (prepare, workflow):
        if not required.is_file():
            raise SystemExit(f"required authoring file does not exist: {required}")

    if prompt.is_file():
        print(f"Using texture prompt: {prompt}")
    else:
        print(f"No texture_prompt.md in {asset_dir}; continuing with image-derived maps")

    subprocess.run([str(authoring_python), str(prepare)], cwd=PROJECT_DIR, check=True)

    started_server: subprocess.Popen[bytes] | None = None
    log_handle = None
    try:
        if not server_ready():
            validation_dir = asset_dir / "validation"
            validation_dir.mkdir(parents=True, exist_ok=True)
            log_handle = (validation_dir / "comfyui.log").open("wb")
            started_server = subprocess.Popen(
                [str(TOOL_DIR / "start.sh")],
                cwd=PROJECT_DIR,
                stdout=log_handle,
                stderr=subprocess.STDOUT,
            )
            deadline = time.monotonic() + 120
            while not server_ready() and time.monotonic() < deadline:
                if started_server.poll() is not None:
                    raise RuntimeError(f"ComfyUI exited; see {validation_dir / 'comfyui.log'}")
                time.sleep(1)
            if not server_ready():
                raise TimeoutError("ComfyUI did not become ready within 120 seconds")

        command = [
            str(authoring_python),
            str(TOOL_DIR / "submit_workflow.py"),
            str(workflow),
            "--input-dir", str(input_dir),
            "--input-namespace", namespace,
            "--output-dir", str(output_dir),
        ]
        if prompt.is_file():
            command += ["--prompt-file", str(prompt)]
        subprocess.run(command, cwd=PROJECT_DIR, check=True)
        if validator.is_file():
            subprocess.run([str(authoring_python), str(validator)], cwd=PROJECT_DIR, check=True)

        for source_name, project_target in config.get("runtime_aliases", {}).items():
            source = resolve(asset_dir, source_name)
            destination = (PROJECT_DIR / project_target).resolve()
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
            print(f"Runtime alias: {destination}")
    finally:
        if started_server is not None:
            started_server.terminate()
            try:
                started_server.wait(timeout=10)
            except subprocess.TimeoutExpired:
                started_server.kill()
        if log_handle is not None:
            log_handle.close()


if __name__ == "__main__":
    main()

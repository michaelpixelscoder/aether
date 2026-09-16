#!/usr/bin/env python3
"""Submit one saved API workflow and collect its named SaveImage outputs."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import time
import urllib.error
import urllib.request
import uuid
from pathlib import Path


def request_json(url: str, payload: dict | None = None) -> dict:
    data = None if payload is None else json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=data,
        headers={"Content-Type": "application/json"},
        method="GET" if data is None else "POST",
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("workflow", type=Path)
    parser.add_argument("--input-dir", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--input-namespace", required=True)
    parser.add_argument("--prompt-file", type=Path)
    parser.add_argument("--server", default="http://127.0.0.1:8188")
    parser.add_argument("--timeout", type=int, default=300)
    args = parser.parse_args()

    tool_dir = Path(__file__).resolve().parent
    project_dir = tool_dir.parents[1]
    comfy_dir = tool_dir / ".runtime" / "ComfyUI"
    comfy_input = comfy_dir / "input" / args.input_namespace
    comfy_input.mkdir(parents=True, exist_ok=True)
    for source in args.input_dir.glob("*.png"):
        shutil.copy2(source, comfy_input / source.name)

    workflow = json.loads(args.workflow.read_text())
    client_id = str(uuid.uuid4())
    prompt_text = args.prompt_file.read_text() if args.prompt_file and args.prompt_file.is_file() else None
    payload = {"prompt": workflow, "client_id": client_id}
    if prompt_text is not None:
        payload["extra_data"] = {
            "aether": {
                "texture_prompt_file": str(args.prompt_file),
                "texture_prompt": prompt_text,
                "texture_prompt_sha256": hashlib.sha256(prompt_text.encode()).hexdigest(),
            }
        }
    queued = request_json(f"{args.server}/prompt", payload)
    prompt_id = queued["prompt_id"]

    deadline = time.monotonic() + args.timeout
    history: dict = {}
    while time.monotonic() < deadline:
        history = request_json(f"{args.server}/history/{prompt_id}")
        if prompt_id in history:
            break
        time.sleep(0.5)
    else:
        raise TimeoutError(f"workflow {prompt_id} did not finish within {args.timeout}s")

    record = history[prompt_id]
    if record.get("status", {}).get("status_str") == "error":
        raise RuntimeError(json.dumps(record.get("status"), indent=2))

    args.output_dir.mkdir(parents=True, exist_ok=True)
    copied: list[str] = []
    for node in record.get("outputs", {}).values():
        for image in node.get("images", []):
            source = comfy_dir / "output" / image.get("subfolder", "") / image["filename"]
            match = re.fullmatch(r"(.+)_\d+_\.png", image["filename"])
            if match is None:
                raise RuntimeError(f"unknown workflow output name: {image['filename']}")
            name = match.group(1) + ".png"
            destination = args.output_dir / name
            shutil.copy2(source, destination)
            copied.append(str(destination))

    run_record = {
        "prompt_id": prompt_id,
        "client_id": client_id,
        "workflow": str(args.workflow.resolve().relative_to(project_dir)),
        "input_namespace": args.input_namespace,
        "texture_prompt_file": str(args.prompt_file.resolve().relative_to(project_dir)) if prompt_text is not None else None,
        "texture_prompt_sha256": hashlib.sha256(prompt_text.encode()).hexdigest() if prompt_text is not None else None,
        "outputs": [str(Path(path).resolve().relative_to(project_dir)) for path in copied],
        "status": record.get("status", {}),
    }
    (args.output_dir / "comfyui-run.json").write_text(json.dumps(run_record, indent=2) + "\n")
    print("\n".join(copied))


if __name__ == "__main__":
    try:
        main()
    except urllib.error.URLError as error:
        raise SystemExit(f"ComfyUI API is unavailable: {error}") from error

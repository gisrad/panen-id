"""Cloudflare R2 uploader for panen-id seasonal bundles."""

import json
import os
from pathlib import Path
from typing import Any

from config import R2_BUCKET, R2_ENDPOINT, R2_PREFIX


def upload_season_artifacts(
    artifacts: dict[str, Any],
    out_dir: Path,
    dry_run: bool = False,
) -> list[str]:
    out_dir.mkdir(parents=True, exist_ok=True)
    keys: list[str] = []
    for rel_name, payload in artifacts.items():
        local = out_dir / rel_name
        local.parent.mkdir(parents=True, exist_ok=True)
        local.write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8")
        keys.append(f"{R2_PREFIX}/{rel_name}".replace("\\", "/"))

    if dry_run:
        return keys

    access_key = os.environ.get("R2_ACCESS_KEY_ID", "")
    secret_key = os.environ.get("R2_SECRET_ACCESS_KEY", "")
    if not (access_key and secret_key and R2_ENDPOINT):
        raise RuntimeError("R2 credentials missing for live season upload.")

    import boto3  # type: ignore

    client = boto3.client(
        "s3",
        endpoint_url=R2_ENDPOINT,
        aws_access_key_id=access_key,
        aws_secret_access_key=secret_key,
        region_name="auto",
    )
    for rel_name, payload in artifacts.items():
        key = f"{R2_PREFIX}/{rel_name}".replace("\\", "/")
        client.put_object(
            Bucket=R2_BUCKET,
            Key=key,
            Body=json.dumps(payload, ensure_ascii=False).encode("utf-8"),
            ContentType="application/json; charset=utf-8",
            CacheControl="public, max-age=3600",
        )
    return keys

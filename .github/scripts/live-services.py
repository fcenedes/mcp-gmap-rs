"""Opt-in, billable read-only probes for the expanded server-side catalog.

Uses the executable's normal configuration. Never creates datasets, queues a
render, submits an optimization job, uploads or publishes imagery. OAuth probes
use list/validation only. Output contains shapes and sanitized errors, not data,
keys, access tokens, playback URLs or session capabilities.
"""

import argparse
import datetime
import json
import os
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[2]
LOCATION = {"latitude": 37.422, "longitude": -122.084}


def probes(mode, project):
    if mode == "oauth":
        return [
            ("maps_datasets", {"operation": "list", "parent": f"projects/{project}", "pageSize": 1}),
            ("maps_route_optimization", {"operation": "validate", "parent": f"projects/{project}", "request": {"model": {"shipments": [], "vehicles": []}}}),
            ("maps_solar", {"operation": "building_insights", "location": LOCATION}),
            ("maps_map_tiles", {"operation": "root_3d"}),
        ]
    future = (datetime.datetime.now(datetime.timezone.utc) + datetime.timedelta(hours=1)).replace(minute=0, second=0, microsecond=0).isoformat().replace("+00:00", "Z")
    return [
        ("maps_autocomplete", {"input": "Googleplex", "includedRegionCodes": ["us"]}),
        ("maps_geolocate", {"body": {"considerIp": True}}),
        ("maps_speed_limits", {"path": [LOCATION]}),
        ("maps_solar", {"operation": "building_insights", "location": LOCATION}),
        ("maps_aerial_view", {"operation": "metadata", "address": "1600 Amphitheatre Parkway, Mountain View, CA"}),
        ("maps_map_tiles", {"operation": "root_3d"}),
        ("maps_streetview", {"operation": "metadata", "location": "37.422,-122.084"}),
        ("maps_streetview", {"operation": "image", "location": "37.422,-122.084", "width": 64, "height": 64}),
        ("maps_air_quality_forecast", {"location": LOCATION, "dateTime": future}),
        ("maps_air_quality_history", {"location": LOCATION, "hours": 1, "pageSize": 1}),
        ("maps_air_quality_heatmap", {"mapType": "UAQI_RED_GREEN", "zoom": 4, "x": 2, "y": 6}),
        ("maps_pollen_heatmap", {"mapType": "TREE_UPI", "zoom": 4, "x": 2, "y": 6}),
        ("maps_weather_history", {"location": LOCATION, "hours": 1, "pageSize": 1}),
        ("maps_weather_alerts", {"location": LOCATION, "pageSize": 1}),
        ("maps_weather_minute_forecast", {"location": LOCATION, "pageSize": 1}),
        ("maps_weather_map", {"mapType": "US_PRECIPITATION_CURRENT", "zoom": 4, "x": 2, "y": 6}),
    ]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--mode", choices=["api-key", "oauth"], default="api-key")
    parser.add_argument("--project", default="")
    parser.add_argument("--credentials", type=Path)
    args = parser.parse_args()
    if args.mode == "oauth" and not args.project:
        parser.error("--project is required for OAuth probes")
    common = [str(args.binary.resolve()), "--auth-mode", args.mode, "--timeout", "20", "--max-retries", "0"]
    environment = os.environ.copy()
    if args.mode == "oauth":
        environment["GOOGLE_CLOUD_QUOTA_PROJECT"] = args.project
        environment["GOOGLE_MAPS_API_KEY"] = ""
        if args.credentials:
            environment["GOOGLE_APPLICATION_CREDENTIALS"] = str(args.credentials.resolve())
    results = []
    for name, payload in probes(args.mode, args.project):
        result = subprocess.run(common + ["call", name, "--args", json.dumps(payload)], cwd=ROOT, env=environment, capture_output=True, text=True, timeout=40)
        row = {"tool": name, "operation": payload.get("operation", ""), "status": "pass" if result.returncode == 0 else "blocked"}
        if result.returncode == 0:
            data = json.loads(result.stdout)
            row["shape"] = sorted(data) if isinstance(data, dict) else type(data).__name__
            if isinstance(data, dict) and "mimeType" in data:
                row["mimeType"] = data["mimeType"]
                row["base64Characters"] = len(data.get("base64", ""))
        else:
            # The executable sanitizes provider errors and never echoes arguments.
            # Exclude trace logs and bound any upstream message.
            row["reason"] = "\n".join(line for line in result.stderr.splitlines() if line.lower().startswith("error:"))[:1200] or "executable rejected the probe; inspect configuration"
        results.append(row)
        print(json.dumps(row, ensure_ascii=False), flush=True)
    print(json.dumps({"mode": args.mode, "passed": sum(row["status"] == "pass" for row in results), "total": len(results)}), flush=True)
    return 0 if all(row["status"] == "pass" for row in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())

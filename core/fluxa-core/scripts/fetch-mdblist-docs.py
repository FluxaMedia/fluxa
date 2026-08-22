#!/usr/bin/env python3
"""Fetch MDBList's OpenAPI spec and render it as a grep-able markdown reference.

Usage: python3 scripts/fetch-mdblist-docs.py
Writes docs/external/mdblist/{openapi.yaml,mdblist-api-docs.md}.
"""
import urllib.request
import yaml
from pathlib import Path

SPEC_URL = "https://api.mdblist.com/schema/"
OUT_DIR = Path(__file__).resolve().parent.parent / "docs" / "external" / "mdblist"


def fetch_spec() -> dict:
    req = urllib.request.Request(SPEC_URL, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req) as resp:
        raw = resp.read()
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    (OUT_DIR / "openapi.yaml").write_bytes(raw)
    return yaml.safe_load(raw)


def resolve(spec: dict, node):
    if isinstance(node, dict) and "$ref" in node:
        ref = node["$ref"]
        assert ref.startswith("#/")
        target = spec
        for part in ref.lstrip("#/").split("/"):
            if not isinstance(target, dict) or part not in target:
                return {"type": ref.rsplit("/", 1)[-1]}
            target = target[part]
        return target
    return node


def schema_summary(spec: dict, schema, depth=0, seen=None) -> str:
    if schema is None:
        return "-"
    seen = seen or set()
    schema = resolve(spec, schema)
    if isinstance(schema, dict) and "$ref" in schema:
        return schema_summary(spec, schema, depth, seen)
    stype = schema.get("type")
    if stype == "object" or "properties" in schema:
        props = schema.get("properties", {})
        if not props:
            return "object"
        if depth > 2:
            return "object { " + ", ".join(sorted(props)) + " }"
        lines = []
        required = set(schema.get("required", []))
        for name, pschema in props.items():
            pschema_r = resolve(spec, pschema)
            mark = "" if name in required else "?"
            lines.append(
                f"{'  ' * (depth + 1)}- `{name}{mark}`: {schema_summary(spec, pschema_r, depth + 1, seen)}"
            )
        return "object {\n" + "\n".join(lines) + f"\n{'  ' * depth}}}"
    if stype == "array":
        items = schema.get("items")
        return f"array<{schema_summary(spec, items, depth, seen)}>"
    if "enum" in schema:
        return f"{stype or 'enum'}({', '.join(str(v) for v in schema['enum'])})"
    return stype or "any"


def render_operation(spec: dict, method: str, path: str, op: dict) -> str:
    lines = [f"### `{method.upper()} {path}`"]
    op_id = op.get("operationId")
    if op_id:
        lines.append(f"operationId: `{op_id}`")
    security = op.get("security", spec.get("security"))
    if security:
        schemes = [name for entry in security for name in entry] or ["none"]
        lines.append(f"auth: {', '.join(sorted(set(schemes))) or 'none'}")
    else:
        lines.append("auth: none")
    summary = op.get("summary") or op.get("description")
    if summary:
        lines.append("")
        lines.append(summary.strip())
    params = op.get("parameters", [])
    if params:
        lines.append("")
        lines.append("Parameters:")
        for p in params:
            p = resolve(spec, p)
            pschema = resolve(spec, p.get("schema", {}))
            req = "required" if p.get("required") else "optional"
            desc = (p.get("description") or "").strip().splitlines()[0] if p.get("description") else ""
            lines.append(
                f"- `{p['name']}` ({p.get('in')}, {req}, {pschema.get('type', 'any')}){': ' + desc if desc else ''}"
            )
    body = op.get("requestBody")
    if body:
        content = body.get("content", {})
        for ctype, cval in content.items():
            lines.append("")
            lines.append(f"Request body ({ctype}):")
            lines.append("```")
            lines.append(schema_summary(spec, cval.get("schema")))
            lines.append("```")
    responses = op.get("responses", {})
    ok = responses.get("200") or responses.get("201") or responses.get("204")
    if ok:
        content = ok.get("content", {})
        for ctype, cval in content.items():
            lines.append("")
            lines.append(f"Response ({ctype}):")
            lines.append("```")
            lines.append(schema_summary(spec, cval.get("schema")))
            lines.append("```")
    lines.append("")
    return "\n".join(lines)


def render(spec: dict) -> str:
    tags = {}
    for path, methods in spec["paths"].items():
        for method, op in methods.items():
            if method not in ("get", "post", "put", "patch", "delete"):
                continue
            for tag in op.get("tags", ["untagged"]):
                tags.setdefault(tag, []).append((method, path, op))

    out = [
        f"# {spec['info']['title']}",
        "",
        f"> Version {spec['info']['version']} - generated from {SPEC_URL} by `scripts/fetch-mdblist-docs.py`.",
        "",
        spec["info"].get("description", "").strip(),
        "",
        "Auth: `apiKey` = `?apikey=YOUR_KEY` query param. `bearerAuth` = `Authorization: Bearer <token>` (user OAuth).",
        "",
        "## Endpoint index",
        "",
    ]
    for tag in sorted(tags):
        out.append(f"- **{tag}**: " + ", ".join(f"`{m.upper()} {p}`" for m, p, _ in tags[tag]))
    out.append("")

    for tag in sorted(tags):
        out.append(f"## {tag}")
        out.append("")
        for method, path, op in sorted(tags[tag], key=lambda t: (t[1], t[0])):
            out.append(render_operation(spec, method, path, op))
    return "\n".join(out)


def main():
    spec = fetch_spec()
    md = render(spec)
    (OUT_DIR / "mdblist-api-docs.md").write_text(md)
    print(f"wrote {OUT_DIR}")


if __name__ == "__main__":
    main()

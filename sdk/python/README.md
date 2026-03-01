# openzax-sdk (Python)

Python SDK for building OpenZax skills.

## Installation

```bash
pip install openzax-sdk
# or with uv
uv add openzax-sdk
```

## Quick start

```python
from openzax import skill, SkillContext, SkillManifest, SkillError

@skill(SkillManifest(
    name="hello-world",
    version="1.0.0",
    description="Greets the caller",
    author="Jane Doe",
    permissions=[],   # list every capability you need
))
def run(ctx: SkillContext, input: dict) -> dict:
    name = input.get("name", "world")
    ctx.info(f"Greeting {name}")
    return {"greeting": f"Hello, {name}!"}
```

## API reference

### `@skill(manifest)` decorator

Register your skill handler. Apply **once** per module.

| Field         | Type        | Description                        |
|---------------|-------------|-------------------------------------|
| `name`        | `str`       | Skill identifier                    |
| `version`     | `str`       | SemVer string                       |
| `description` | `str`       | Human-readable description          |
| `author`      | `str`       | Author name                         |
| `permissions` | `list[str]` | Required capabilities (see below)   |

### `SkillContext`

Injected into every handler call. All methods call through to the host ABI.

| Method | Description |
|--------|-------------|
| `log(level, msg)` | Structured logging (`"trace"`\|`"debug"`\|`"info"`\|`"warn"`\|`"error"`) |
| `trace/debug/info/warn/error(msg)` | Shorthand log methods |
| `get_config(key)` → `str \| None` | Read a configuration value |
| `set_config(key, value)` | Write a configuration value |
| `read_file(path)` → `bytes` | Read a file |
| `write_file(path, data)` | Write a file |
| `http_fetch(url, method, headers, body)` → `HttpResponse` | Make an HTTP request |
| `kv_get(key)` → `str \| None` | Read from key-value store |
| `kv_put(key, value)` | Write to key-value store |
| `kv_delete(key)` | Delete from key-value store |
| `emit_event(name, data)` | Emit a structured event |

### `HttpResponse`

```python
@dataclass
class HttpResponse:
    status: int
    headers: dict[str, str]
    body: bytes
```

Use `decode_response_json(response)` to parse the body as JSON, or
`response.body.decode("utf-8")` for raw text.

### `SkillError`

```python
raise SkillError("NOT_FOUND", "Item does not exist", retryable=False)
```

Caught by the runtime and serialised to `{"error": "NOT_FOUND", "message": "..."}`.

### Utilities

```python
from openzax import decode_response_json, require_ok

resp = ctx.http_fetch("https://api.example.com/data")
require_ok(resp, url="https://api.example.com/data")   # raises SkillError on 4xx/5xx
data = decode_response_json(resp)                       # parse JSON body
```

## Permissions

Declare all capabilities in `SkillManifest.permissions`:

| Permission   | Capability               |
|--------------|--------------------------|
| `fs:read`    | Read files               |
| `fs:write`   | Write files              |
| `net:fetch`  | Make HTTP requests       |
| `kv:read`    | Read from KV store       |
| `kv:write`   | Write to KV store        |

Skills that use a capability without declaring it receive a permission error.

## Testing

Use `set_mock_host` to replace host functions with in-memory mocks:

```python
import pytest
from openzax import set_mock_host, LOG_INFO

_logs: list[tuple[int, str]] = []
_kv: dict[str, str] = {}
_mem = bytearray(64 * 1024)  # 64 KiB scratch

def mock_log(level, ptr, length):
    _logs.append((level, bytes(_mem[ptr:ptr+length]).decode()))

def mock_kv_get(kptr, klen, optr, ocap):
    key = bytes(_mem[kptr:kptr+klen]).decode()
    val = _kv.get(key)
    if val is None:
        return -1
    encoded = val.encode()
    _mem[optr:optr+len(encoded)] = encoded
    return len(encoded)

def mock_kv_put(kptr, klen, vptr, vlen):
    key = bytes(_mem[kptr:kptr+klen]).decode()
    val = bytes(_mem[vptr:vptr+vlen]).decode()
    _kv[key] = val

set_mock_host({
    "__openzax_log":     mock_log,
    "__openzax_kv_get":  mock_kv_get,
    "__openzax_kv_put":  mock_kv_put,
    # ... add other mocks as needed
})
```

For a complete testing example, see the
[test harness documentation](../../crates/test-harness/README.md).

## Building for WASM

```bash
# Install py2wasm or use the official compiler
pip install py2wasm

# Compile skill to WASM
py2wasm my_skill.py -o my_skill.wasm

# Pack into an .ozskill bundle
openzax skill pack .
```

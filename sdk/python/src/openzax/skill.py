"""
OpenZax Python skill SDK.

Define a skill by decorating a function with the :func:`skill` decorator:

.. code-block:: python

    from openzax.skill import skill, SkillContext, SkillManifest, SkillError

    @skill(SkillManifest(
        name="hello-world",
        version="1.0.0",
        description="Greets the caller",
        author="Jane Doe",
        permissions=[],
    ))
    def run(ctx: SkillContext, input: dict) -> dict:
        ctx.info(f"Hello, {input.get('name', 'world')}!")
        return {"greeting": f"Hello, {input.get('name', 'world')}!"}
"""

from __future__ import annotations

import json
import traceback
from dataclasses import dataclass, field
from typing import Any, Callable, Optional

from openzax import host as _host

# ── Public data types ─────────────────────────────────────────────────────────


@dataclass
class HttpResponse:
    """HTTP response returned by :meth:`SkillContext.http_fetch`."""
    status: int
    headers: dict[str, str]
    body: bytes


@dataclass
class SkillManifest:
    """Metadata declaration for a skill."""
    name: str
    version: str
    description: str
    author: str
    permissions: list[str] = field(default_factory=list)


# ── SkillError ────────────────────────────────────────────────────────────────


class SkillError(Exception):
    """
    Structured error raised by skill logic.

    :param code: Machine-readable error code (e.g. ``"NOT_FOUND"``).
    :param message: Human-readable error message.
    :param retryable: Whether the caller may safely retry the invocation.
    """

    def __init__(self, code: str, message: str, retryable: bool = False) -> None:
        super().__init__(message)
        self.code = code
        self.retryable = retryable

    def to_dict(self) -> dict[str, Any]:
        return {
            "error": self.code,
            "message": str(self),
            "retryable": self.retryable,
        }

    def __repr__(self) -> str:
        return f"SkillError(code={self.code!r}, message={str(self)!r}, retryable={self.retryable})"


# ── SkillContext ──────────────────────────────────────────────────────────────


class SkillContext:
    """
    Runtime context injected into every skill invocation.

    Provides permission-gated access to host capabilities.
    All methods call through to the host ABI defined in :mod:`openzax.host`.
    """

    # ── Logging ───────────────────────────────────────────────────────────────

    def log(self, level: str, message: str) -> None:
        """Emit a structured log message at ``level``."""
        level_map = {
            "trace": _host.LOG_TRACE,
            "debug": _host.LOG_DEBUG,
            "info":  _host.LOG_INFO,
            "warn":  _host.LOG_WARN,
            "error": _host.LOG_ERROR,
        }
        _host.host_log(level_map.get(level, _host.LOG_INFO), message)

    def trace(self, message: str) -> None:
        self.log("trace", message)

    def debug(self, message: str) -> None:
        self.log("debug", message)

    def info(self, message: str) -> None:
        self.log("info", message)

    def warn(self, message: str) -> None:
        self.log("warn", message)

    def error(self, message: str) -> None:
        self.log("error", message)

    # ── Config ────────────────────────────────────────────────────────────────

    def get_config(self, key: str) -> Optional[str]:
        """Read a configuration value, or ``None`` if not set."""
        return _host.host_config_get(key)

    def set_config(self, key: str, value: str) -> None:
        """Write a configuration value."""
        _host.host_config_set(key, value)

    # ── Filesystem ────────────────────────────────────────────────────────────

    def read_file(self, path: str) -> bytes:
        """Read a file from the virtual filesystem.

        :raises FileNotFoundError: if the file does not exist.
        :raises PermissionError: if ``fs:read`` is not in the skill's permissions.
        """
        return _host.host_read_file(path)

    def write_file(self, path: str, data: bytes) -> None:
        """Write a file to the virtual filesystem.

        :raises PermissionError: if ``fs:write`` is not in the skill's permissions.
        """
        _host.host_write_file(path, data)

    # ── HTTP ──────────────────────────────────────────────────────────────────

    def http_fetch(
        self,
        url: str,
        method: str = "GET",
        headers: dict[str, str] | None = None,
        body: bytes = b"",
    ) -> HttpResponse:
        """Make an HTTP request.

        :raises PermissionError: if ``net:fetch`` is not in the skill's permissions.
        :raises ConnectionError: if the request failed at the transport level.
        """
        raw = _host.host_http_fetch(url, method, headers or {}, body)
        return HttpResponse(
            status=int(raw.get("status", 0)),
            headers={k: str(v) for k, v in (raw.get("headers") or {}).items()},
            body=bytes(raw.get("body") or b""),
        )

    # ── KV store ──────────────────────────────────────────────────────────────

    def kv_get(self, key: str) -> Optional[str]:
        """Read a value from the key-value store, or ``None`` if absent."""
        return _host.host_kv_get(key)

    def kv_put(self, key: str, value: str) -> None:
        """Write a value to the key-value store."""
        _host.host_kv_put(key, value)

    def kv_delete(self, key: str) -> None:
        """Delete a key from the key-value store."""
        _host.host_kv_delete(key)

    # ── Events ────────────────────────────────────────────────────────────────

    def emit_event(self, name: str, data: Any) -> None:
        """Emit a named event with a JSON-serialisable payload."""
        _host.host_emit_event(name, data)


# ── Type alias ────────────────────────────────────────────────────────────────

SkillHandler = Callable[[SkillContext, Any], Any]

# ── Internal registry ─────────────────────────────────────────────────────────

_registered_skill: Optional[tuple[SkillManifest, SkillHandler]] = None


# ── Public decorator ─────────────────────────────────────────────────────────


def skill(manifest: SkillManifest) -> Callable[[SkillHandler], SkillHandler]:
    """
    Class / function decorator to register a skill handler.

    .. code-block:: python

        @skill(SkillManifest(name="my-skill", version="0.1.0",
                             description="...", author="..."))
        def run(ctx: SkillContext, input: dict) -> dict:
            ...
    """
    def decorator(func: SkillHandler) -> SkillHandler:
        global _registered_skill
        if _registered_skill is not None:
            raise SkillError(
                "DOUBLE_REGISTER",
                "skill() decorator was applied more than once. "
                "Only one skill handler can be registered per module.",
            )
        _registered_skill = (manifest, func)
        return func

    return decorator


# ── WASM entry point ──────────────────────────────────────────────────────────


def __openzax_skill_call(input_json: str) -> str:
    """
    Entry point called by the OpenZax runtime for every skill invocation.

    ``input_json``  – JSON-encoded input (UTF-8 string).
    Returns          – JSON-encoded output or ``{"error": ..., "message": ...}``.
    """
    if _registered_skill is None:
        err = SkillError(
            "NO_SKILL",
            "No skill has been registered. Apply the @skill() decorator.",
        )
        return json.dumps(err.to_dict())

    manifest, handler = _registered_skill
    ctx = SkillContext()

    try:
        input_data: Any = json.loads(input_json)
    except json.JSONDecodeError as exc:
        err = SkillError("INVALID_INPUT", f"Input is not valid JSON: {exc}")
        return json.dumps(err.to_dict())

    try:
        output = handler(ctx, input_data)
        return json.dumps(output)
    except SkillError as exc:
        return json.dumps(exc.to_dict())
    except Exception as exc:
        tb = traceback.format_exc()
        ctx.error(f"Unhandled exception in skill '{manifest.name}': {tb}")
        err = SkillError("RUNTIME_ERROR", str(exc))
        return json.dumps(err.to_dict())


# ── Utility helpers ───────────────────────────────────────────────────────────


def decode_response_json(response: HttpResponse) -> Any:
    """Parse the body of an :class:`HttpResponse` as JSON."""
    try:
        return json.loads(response.body.decode("utf-8"))
    except (json.JSONDecodeError, UnicodeDecodeError) as exc:
        raise SkillError(
            "INVALID_RESPONSE",
            f"Failed to parse JSON response: {exc}",
        ) from exc


def require_ok(response: HttpResponse, url: str = "") -> HttpResponse:
    """Raise :class:`SkillError` if the HTTP response status is not 2xx."""
    if not 200 <= response.status < 300:
        raise SkillError(
            "HTTP_ERROR",
            f"HTTP {response.status} for {url or '(url)'}",
            retryable=response.status >= 500,
        )
    return response

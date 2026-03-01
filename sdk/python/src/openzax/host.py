"""
Low-level WASM host function bindings using ctypes.

When a Python skill is compiled to WASM (e.g. via py2wasm or CPython WASM),
the host provides these imports. This module wraps them so that
:mod:`openzax.skill` can call host functions without caring about the
underlying ABI.

In a native (non-WASM) test environment, call :func:`set_mock_host` to
replace the bindings with in-memory mocks.
"""

from __future__ import annotations

import ctypes
import json
from typing import Any, Callable

# ── Types ─────────────────────────────────────────────────────────────────────

# Signature of a raw host function: (ptr, len, ...) -> int
HostFn = Callable[..., int]

# ── Log level constants ───────────────────────────────────────────────────────

LOG_TRACE = 0
LOG_DEBUG = 1
LOG_INFO = 2
LOG_WARN = 3
LOG_ERROR = 4

# ── Global function table ─────────────────────────────────────────────────────

_FUNCTIONS: dict[str, HostFn] = {}

# Current WASM linear memory (set by __set_memory in WASM environments).
_memory: bytearray | None = None

# Output scratch buffer size (1 MiB)
_OUT_BUF_SIZE = 1024 * 1024


def set_mock_host(functions: dict[str, HostFn]) -> None:
    """Replace host bindings with mock functions (for testing)."""
    global _FUNCTIONS
    _FUNCTIONS = dict(functions)


def set_memory(mem: bytearray) -> None:
    """Provide access to WASM linear memory (called by the WASM runtime)."""
    global _memory
    _memory = mem


def _fn(name: str) -> HostFn:
    fn = _FUNCTIONS.get(name)
    if fn is None:
        raise RuntimeError(
            f"OpenZax host function '{name}' is not available. "
            "Call set_mock_host() in tests, or run inside the OpenZax runtime."
        )
    return fn


# ── Memory helpers ────────────────────────────────────────────────────────────

def _mem() -> bytearray:
    if _memory is None:
        raise RuntimeError(
            "WASM memory is not initialised. "
            "Call set_memory() or use set_mock_host() for testing."
        )
    return _memory


def _write_str(s: str) -> tuple[int, int]:
    """Write UTF-8 bytes into WASM memory via __openzax_alloc. Returns (ptr, len)."""
    data = s.encode("utf-8")
    ptr = _fn("__openzax_alloc")(len(data))
    if not ptr:
        raise MemoryError("__openzax_alloc returned null")
    _mem()[ptr : ptr + len(data)] = data
    return ptr, len(data)


def _write_bytes(data: bytes) -> tuple[int, int]:
    ptr = _fn("__openzax_alloc")(len(data))
    if not ptr:
        raise MemoryError("__openzax_alloc returned null")
    _mem()[ptr : ptr + len(data)] = data
    return ptr, len(data)


def _read_str(ptr: int, length: int) -> str:
    return bytes(_mem()[ptr : ptr + length]).decode("utf-8")


def _read_bytes(ptr: int, length: int) -> bytes:
    return bytes(_mem()[ptr : ptr + length])


def _alloc_out() -> tuple[int, int]:
    """Allocate a scratch output buffer. Returns (ptr, cap)."""
    ptr = _fn("__openzax_alloc")(_OUT_BUF_SIZE)
    return ptr, _OUT_BUF_SIZE


def _free(ptr: int, length: int) -> None:
    try:
        _fn("__openzax_free")(ptr, length)
    except Exception:
        pass


# ── Public host call wrappers ─────────────────────────────────────────────────


def host_log(level: int, message: str) -> None:
    ptr, length = _write_str(message)
    _fn("__openzax_log")(level, ptr, length)
    _free(ptr, length)


def host_config_get(key: str) -> str | None:
    kptr, klen = _write_str(key)
    optr, ocap = _alloc_out()
    written = _fn("__openzax_config_get")(kptr, klen, optr, ocap)
    _free(kptr, klen)
    if written < 0:
        _free(optr, ocap)
        return None
    value = _read_str(optr, written)
    _free(optr, ocap)
    return value


def host_config_set(key: str, value: str) -> None:
    kptr, klen = _write_str(key)
    vptr, vlen = _write_str(value)
    _fn("__openzax_config_set")(kptr, klen, vptr, vlen)
    _free(kptr, klen)
    _free(vptr, vlen)


def host_read_file(path: str) -> bytes:
    pptr, plen = _write_str(path)
    optr, ocap = _alloc_out()
    written = _fn("__openzax_read_file")(pptr, plen, optr, ocap)
    _free(pptr, plen)
    if written < 0:
        _free(optr, ocap)
        raise FileNotFoundError(f"File not found: {path}")
    data = _read_bytes(optr, written)
    _free(optr, ocap)
    return data


def host_write_file(path: str, data: bytes) -> None:
    pptr, plen = _write_str(path)
    dptr, dlen = _write_bytes(data)
    result = _fn("__openzax_write_file")(pptr, plen, dptr, dlen)
    _free(pptr, plen)
    _free(dptr, dlen)
    if result < 0:
        raise OSError(f"Failed to write file: {path}")


def host_http_fetch(
    url: str,
    method: str,
    headers: dict[str, str],
    body: bytes,
) -> dict[str, Any]:
    uptr, ulen = _write_str(url)
    mptr, mlen = _write_str(method)
    hptr, hlen = _write_str(json.dumps(headers))
    bptr, blen = _write_bytes(body)
    optr, ocap = _alloc_out()

    written = _fn("__openzax_http_fetch")(
        uptr, ulen,
        mptr, mlen,
        hptr, hlen,
        bptr, blen,
        optr, ocap,
    )

    _free(uptr, ulen)
    _free(mptr, mlen)
    _free(hptr, hlen)
    _free(bptr, blen)

    if written < 0:
        _free(optr, ocap)
        raise ConnectionError(f"HTTP fetch failed for {method} {url}")

    raw = _read_str(optr, written)
    _free(optr, ocap)
    return json.loads(raw)  # type: ignore[no-any-return]


def host_kv_get(key: str) -> str | None:
    kptr, klen = _write_str(key)
    optr, ocap = _alloc_out()
    written = _fn("__openzax_kv_get")(kptr, klen, optr, ocap)
    _free(kptr, klen)
    if written < 0:
        _free(optr, ocap)
        return None
    value = _read_str(optr, written)
    _free(optr, ocap)
    return value


def host_kv_put(key: str, value: str) -> None:
    kptr, klen = _write_str(key)
    vptr, vlen = _write_str(value)
    _fn("__openzax_kv_put")(kptr, klen, vptr, vlen)
    _free(kptr, klen)
    _free(vptr, vlen)


def host_kv_delete(key: str) -> None:
    kptr, klen = _write_str(key)
    _fn("__openzax_kv_delete")(kptr, klen)
    _free(kptr, klen)


def host_emit_event(name: str, data: Any) -> None:
    nptr, nlen = _write_str(name)
    dptr, dlen = _write_str(json.dumps(data))
    _fn("__openzax_emit_event")(nptr, nlen, dptr, dlen)
    _free(nptr, nlen)
    _free(dptr, dlen)


# ── ctypes-based native loader (non-WASM environments) ───────────────────────

def load_native_host(library_path: str) -> None:
    """
    Load a native shared library that exports the OpenZax host ABI and
    register its functions. Useful when running skills natively for testing.

    The shared library must export symbols matching the __openzax_* names with
    the C calling convention.
    """
    lib = ctypes.CDLL(library_path)

    def make_fn(name: str, restype: Any, *argtypes: Any) -> HostFn:
        fn = getattr(lib, name, None)
        if fn is None:
            return lambda *_: -1
        fn.restype = restype
        fn.argtypes = list(argtypes)
        return fn  # type: ignore[return-value]

    i32 = ctypes.c_int32
    void = None

    set_mock_host({
        "__openzax_log":          make_fn("__openzax_log", void, i32, i32, i32),
        "__openzax_config_get":   make_fn("__openzax_config_get", i32, i32, i32, i32, i32),
        "__openzax_config_set":   make_fn("__openzax_config_set", void, i32, i32, i32, i32),
        "__openzax_read_file":    make_fn("__openzax_read_file", i32, i32, i32, i32, i32),
        "__openzax_write_file":   make_fn("__openzax_write_file", i32, i32, i32, i32, i32),
        "__openzax_http_fetch":   make_fn("__openzax_http_fetch", i32,
                                          i32, i32, i32, i32, i32, i32, i32, i32, i32, i32),
        "__openzax_kv_get":       make_fn("__openzax_kv_get", i32, i32, i32, i32, i32),
        "__openzax_kv_put":       make_fn("__openzax_kv_put", void, i32, i32, i32, i32),
        "__openzax_kv_delete":    make_fn("__openzax_kv_delete", void, i32, i32),
        "__openzax_emit_event":   make_fn("__openzax_emit_event", void, i32, i32, i32, i32),
        "__openzax_alloc":        make_fn("__openzax_alloc", i32, i32),
        "__openzax_free":         make_fn("__openzax_free", void, i32, i32),
    })

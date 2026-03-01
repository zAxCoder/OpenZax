/**
 * Low-level bindings to the OpenZax WASM host ABI.
 *
 * These functions are *imported* from the embedding host at link time.
 * When compiled to WASM via AssemblyScript or similar, the host fills in
 * real implementations. During unit testing the host stubs can be replaced
 * with mocks.
 *
 * Memory layout convention:
 *   - Strings / byte buffers are passed as (ptr: i32, len: i32) pairs
 *     pointing into the module's linear memory.
 *   - Output buffers are caller-allocated; the host writes at most `cap`
 *     bytes and returns the actual length, or -1 on miss / error.
 */

// ── Log level constants ───────────────────────────────────────────────────────

export const LOG_TRACE = 0;
export const LOG_DEBUG = 1;
export const LOG_INFO  = 2;
export const LOG_WARN  = 3;
export const LOG_ERROR = 4;

// ── Host function declarations ────────────────────────────────────────────────
// These are filled in by the WASM linker at instantiation time.

// eslint-disable-next-line @typescript-eslint/no-namespace
declare namespace HostImports {
  /** Emit a structured log message. */
  function __openzax_log(level: number, msgPtr: number, msgLen: number): void;

  /** Read a configuration value. Returns written byte count or -1. */
  function __openzax_config_get(
    keyPtr: number, keyLen: number,
    outPtr: number, outCap: number,
  ): number;

  /** Write a configuration value. */
  function __openzax_config_set(
    keyPtr: number, keyLen: number,
    valPtr: number, valLen: number,
  ): void;

  /** Read a file from the virtual filesystem. Returns byte count or -1. */
  function __openzax_read_file(
    pathPtr: number, pathLen: number,
    outPtr: number, outCap: number,
  ): number;

  /** Write a file to the virtual filesystem. Returns 0 on success, -1 on error. */
  function __openzax_write_file(
    pathPtr: number, pathLen: number,
    dataPtr: number, dataLen: number,
  ): number;

  /**
   * Make an HTTP request.
   * `headersPtr` points to a JSON-encoded `Record<string,string>`.
   * The response is written as JSON `{status, headers, body}` at `outPtr`.
   * Returns byte count written or -1 on error.
   */
  function __openzax_http_fetch(
    urlPtr: number, urlLen: number,
    methodPtr: number, methodLen: number,
    headersPtr: number, headersLen: number,
    bodyPtr: number, bodyLen: number,
    outPtr: number, outCap: number,
  ): number;

  /** Read a KV value. Returns byte count or -1. */
  function __openzax_kv_get(
    keyPtr: number, keyLen: number,
    outPtr: number, outCap: number,
  ): number;

  /** Write a KV entry. */
  function __openzax_kv_put(
    keyPtr: number, keyLen: number,
    valPtr: number, valLen: number,
  ): void;

  /** Delete a KV entry. */
  function __openzax_kv_delete(keyPtr: number, keyLen: number): void;

  /** Emit an event.  `dataPtr` points to a JSON-encoded payload. */
  function __openzax_emit_event(
    namePtr: number, nameLen: number,
    dataPtr: number, dataLen: number,
  ): void;

  /** Allocate `len` bytes in WASM linear memory. Returns pointer or 0. */
  function __openzax_alloc(len: number): number;

  /** Free memory previously allocated via __openzax_alloc. */
  function __openzax_free(ptr: number, len: number): void;
}

// ── Helpers for environments that resolve imports dynamically ─────────────────

/**
 * Resolve a host function by name.  In real WASM builds, the linker provides
 * these; this shim makes the SDK usable in Node.js / testing environments.
 */
let _imports: Record<string, (...args: number[]) => number | void> = {};

export function __setHostImports(
  imports: Record<string, (...args: number[]) => number | void>,
): void {
  _imports = imports;
}

function host(name: string): (...args: number[]) => number | void {
  const fn_ = _imports[name];
  if (!fn_) {
    throw new Error(
      `OpenZax host function '${name}' is not available. ` +
      `Did you call __setHostImports()?`,
    );
  }
  return fn_;
}

// ── Memory helper ─────────────────────────────────────────────────────────────

let _memory: WebAssembly.Memory | null = null;
const _textEncoder = typeof TextEncoder !== 'undefined' ? new TextEncoder() : null;
const _textDecoder = typeof TextDecoder !== 'undefined' ? new TextDecoder() : null;

export function __setMemory(mem: WebAssembly.Memory): void {
  _memory = mem;
}

function mem(): DataView {
  if (!_memory) {
    throw new Error('WASM memory not initialised. Call __setMemory().');
  }
  return new DataView(_memory.buffer);
}

function memU8(): Uint8Array {
  if (!_memory) throw new Error('WASM memory not initialised.');
  return new Uint8Array(_memory.buffer);
}

/** Allocate + write a UTF-8 string. Returns [ptr, len]. */
export function writeString(s: string): [number, number] {
  const encoded = _textEncoder
    ? _textEncoder.encode(s)
    : Buffer.from(s, 'utf8');
  const ptr = host('__openzax_alloc')(encoded.length) as number;
  if (!ptr) throw new Error('__openzax_alloc returned null');
  memU8().set(encoded, ptr);
  return [ptr, encoded.length];
}

/** Read a UTF-8 string at (ptr, len). */
export function readString(ptr: number, len: number): string {
  const slice = memU8().slice(ptr, ptr + len);
  return _textDecoder
    ? _textDecoder.decode(slice)
    : Buffer.from(slice).toString('utf8');
}

/** Allocate + write raw bytes. Returns [ptr, len]. */
export function writeBytes(data: Uint8Array): [number, number] {
  const ptr = host('__openzax_alloc')(data.length) as number;
  if (!ptr) throw new Error('__openzax_alloc returned null');
  memU8().set(data, ptr);
  return [ptr, data.length];
}

/** Read bytes at (ptr, len). */
export function readBytes(ptr: number, len: number): Uint8Array {
  return memU8().slice(ptr, ptr + len);
}

// ── Public host call wrappers ─────────────────────────────────────────────────

const OUT_BUFFER_CAP = 1024 * 1024; // 1 MiB output buffer

/** Allocate a scratch output buffer. Returns [ptr, cap]. */
function allocOutBuffer(): [number, number] {
  const ptr = host('__openzax_alloc')(OUT_BUFFER_CAP) as number;
  return [ptr, OUT_BUFFER_CAP];
}

export function hostLog(level: number, message: string): void {
  const [ptr, len] = writeString(message);
  host('__openzax_log')(level, ptr, len);
  host('__openzax_free')(ptr, len);
}

export function hostConfigGet(key: string): string | undefined {
  const [kPtr, kLen] = writeString(key);
  const [outPtr, outCap] = allocOutBuffer();
  const written = host('__openzax_config_get')(kPtr, kLen, outPtr, outCap) as number;
  host('__openzax_free')(kPtr, kLen);
  if (written < 0) {
    host('__openzax_free')(outPtr, outCap);
    return undefined;
  }
  const value = readString(outPtr, written);
  host('__openzax_free')(outPtr, outCap);
  return value;
}

export function hostConfigSet(key: string, value: string): void {
  const [kPtr, kLen] = writeString(key);
  const [vPtr, vLen] = writeString(value);
  host('__openzax_config_set')(kPtr, kLen, vPtr, vLen);
  host('__openzax_free')(kPtr, kLen);
  host('__openzax_free')(vPtr, vLen);
}

export function hostReadFile(path: string): Uint8Array {
  const [pPtr, pLen] = writeString(path);
  const [outPtr, outCap] = allocOutBuffer();
  const written = host('__openzax_read_file')(pPtr, pLen, outPtr, outCap) as number;
  host('__openzax_free')(pPtr, pLen);
  if (written < 0) {
    host('__openzax_free')(outPtr, outCap);
    throw new Error(`File not found: ${path}`);
  }
  const data = readBytes(outPtr, written);
  host('__openzax_free')(outPtr, outCap);
  return data;
}

export function hostWriteFile(path: string, data: Uint8Array): void {
  const [pPtr, pLen] = writeString(path);
  const [dPtr, dLen] = writeBytes(data);
  const result = host('__openzax_write_file')(pPtr, pLen, dPtr, dLen) as number;
  host('__openzax_free')(pPtr, pLen);
  host('__openzax_free')(dPtr, dLen);
  if (result < 0) throw new Error(`Failed to write file: ${path}`);
}

export function hostHttpFetch(
  url: string,
  method: string,
  headers: Record<string, string>,
  body?: Uint8Array,
): { status: number; headers: Record<string, string>; body: Uint8Array } {
  const [urlPtr, urlLen] = writeString(url);
  const [mPtr, mLen] = writeString(method);
  const headersJson = JSON.stringify(headers);
  const [hPtr, hLen] = writeString(headersJson);
  const bodyData = body ?? new Uint8Array(0);
  const [bPtr, bLen] = writeBytes(bodyData);
  const [outPtr, outCap] = allocOutBuffer();

  const written = host('__openzax_http_fetch')(
    urlPtr, urlLen,
    mPtr, mLen,
    hPtr, hLen,
    bPtr, bLen,
    outPtr, outCap,
  ) as number;

  host('__openzax_free')(urlPtr, urlLen);
  host('__openzax_free')(mPtr, mLen);
  host('__openzax_free')(hPtr, hLen);
  host('__openzax_free')(bPtr, bLen);

  if (written < 0) {
    host('__openzax_free')(outPtr, outCap);
    throw new Error(`HTTP fetch failed for ${method} ${url}`);
  }

  const responseJson = readString(outPtr, written);
  host('__openzax_free')(outPtr, outCap);

  const parsed = JSON.parse(responseJson) as {
    status: number;
    headers: Record<string, string>;
    body: number[];
  };

  return {
    status: parsed.status,
    headers: parsed.headers ?? {},
    body: Uint8Array.from(parsed.body ?? []),
  };
}

export function hostKvGet(key: string): string | undefined {
  const [kPtr, kLen] = writeString(key);
  const [outPtr, outCap] = allocOutBuffer();
  const written = host('__openzax_kv_get')(kPtr, kLen, outPtr, outCap) as number;
  host('__openzax_free')(kPtr, kLen);
  if (written < 0) {
    host('__openzax_free')(outPtr, outCap);
    return undefined;
  }
  const value = readString(outPtr, written);
  host('__openzax_free')(outPtr, outCap);
  return value;
}

export function hostKvPut(key: string, value: string): void {
  const [kPtr, kLen] = writeString(key);
  const [vPtr, vLen] = writeString(value);
  host('__openzax_kv_put')(kPtr, kLen, vPtr, vLen);
  host('__openzax_free')(kPtr, kLen);
  host('__openzax_free')(vPtr, vLen);
}

export function hostKvDelete(key: string): void {
  const [kPtr, kLen] = writeString(key);
  host('__openzax_kv_delete')(kPtr, kLen);
  host('__openzax_free')(kPtr, kLen);
}

export function hostEmitEvent(name: string, data: unknown): void {
  const [nPtr, nLen] = writeString(name);
  const dataJson = JSON.stringify(data);
  const [dPtr, dLen] = writeString(dataJson);
  host('__openzax_emit_event')(nPtr, nLen, dPtr, dLen);
  host('__openzax_free')(nPtr, nLen);
  host('__openzax_free')(dPtr, dLen);
}

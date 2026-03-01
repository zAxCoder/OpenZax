import {
  LOG_DEBUG,
  LOG_ERROR,
  LOG_INFO,
  LOG_TRACE,
  LOG_WARN,
  hostConfigGet,
  hostConfigSet,
  hostEmitEvent,
  hostHttpFetch,
  hostKvDelete,
  hostKvGet,
  hostKvPut,
  hostLog,
  hostReadFile,
  hostWriteFile,
} from './host-bindings';

// ── Public types ──────────────────────────────────────────────────────────────

export interface HttpResponse {
  status: number;
  headers: Record<string, string>;
  body: Uint8Array;
}

export interface SkillContext {
  /** Write a structured log message at the given level. */
  log(level: 'trace' | 'debug' | 'info' | 'warn' | 'error', message: string): void;
  /** Convenience log helpers. */
  trace(msg: string): void;
  debug(msg: string): void;
  info(msg: string): void;
  warn(msg: string): void;
  error(msg: string): void;

  getConfig(key: string): string | undefined;
  setConfig(key: string, value: string): void;

  readFile(path: string): Uint8Array;
  writeFile(path: string, data: Uint8Array): void;

  httpFetch(
    url: string,
    method: string,
    headers: Record<string, string>,
    body?: Uint8Array,
  ): HttpResponse;

  kvGet(key: string): string | undefined;
  kvPut(key: string, value: string): void;
  kvDelete(key: string): void;

  emitEvent(name: string, data: unknown): void;
}

export interface SkillManifest {
  name: string;
  version: string;
  description: string;
  author: string;
  permissions: string[];
}

export type SkillHandler = (
  ctx: SkillContext,
  input: unknown,
) => Promise<unknown> | unknown;

// ── SkillError ────────────────────────────────────────────────────────────────

export class SkillError extends Error {
  public readonly code: string;
  public readonly retryable: boolean;

  constructor(code: string, message: string, retryable = false) {
    super(message);
    this.name = 'SkillError';
    this.code = code;
    this.retryable = retryable;
    // Preserve prototype chain in environments that transpile classes
    Object.setPrototypeOf(this, SkillError.prototype);
  }

  toJSON(): Record<string, unknown> {
    return {
      error: this.code,
      message: this.message,
      retryable: this.retryable,
    };
  }
}

// ── SkillContext implementation ────────────────────────────────────────────────

class DefaultSkillContext implements SkillContext {
  log(level: 'trace' | 'debug' | 'info' | 'warn' | 'error', message: string): void {
    const levelNum = { trace: LOG_TRACE, debug: LOG_DEBUG, info: LOG_INFO, warn: LOG_WARN, error: LOG_ERROR }[level];
    hostLog(levelNum, message);
  }

  trace(msg: string): void { this.log('trace', msg); }
  debug(msg: string): void { this.log('debug', msg); }
  info(msg: string): void  { this.log('info', msg);  }
  warn(msg: string): void  { this.log('warn', msg);  }
  error(msg: string): void { this.log('error', msg); }

  getConfig(key: string): string | undefined {
    return hostConfigGet(key);
  }

  setConfig(key: string, value: string): void {
    hostConfigSet(key, value);
  }

  readFile(path: string): Uint8Array {
    return hostReadFile(path);
  }

  writeFile(path: string, data: Uint8Array): void {
    hostWriteFile(path, data);
  }

  httpFetch(
    url: string,
    method = 'GET',
    headers: Record<string, string> = {},
    body?: Uint8Array,
  ): HttpResponse {
    return hostHttpFetch(url, method, headers, body);
  }

  kvGet(key: string): string | undefined {
    return hostKvGet(key);
  }

  kvPut(key: string, value: string): void {
    hostKvPut(key, value);
  }

  kvDelete(key: string): void {
    hostKvDelete(key);
  }

  emitEvent(name: string, data: unknown): void {
    hostEmitEvent(name, data);
  }
}

// ── Skill registry ────────────────────────────────────────────────────────────

interface RegisteredSkill {
  manifest: SkillManifest;
  handler: SkillHandler;
}

let _registeredSkill: RegisteredSkill | null = null;

/**
 * Register a skill handler.
 *
 * Call this exactly once at module initialisation. The OpenZax runtime will
 * call the exported `__openzax_skill_call` function on each invocation.
 *
 * @example
 * ```typescript
 * import { defineSkill, SkillContext } from '@openzax/sdk';
 *
 * defineSkill(
 *   { name: 'my-skill', version: '1.0.0', description: '...', author: '...', permissions: [] },
 *   async (ctx: SkillContext, input: unknown) => {
 *     ctx.info('Running my skill');
 *     return { ok: true };
 *   },
 * );
 * ```
 */
export function defineSkill(manifest: SkillManifest, handler: SkillHandler): void {
  if (_registeredSkill) {
    throw new SkillError(
      'DOUBLE_REGISTER',
      'defineSkill() was called more than once. Only one skill can be registered per WASM module.',
    );
  }
  _registeredSkill = { manifest, handler };
}

// ── WASM export: __openzax_skill_call ─────────────────────────────────────────

/**
 * Entry point exported from the WASM module.
 *
 * The host calls this function with:
 *   - `inputPtr` / `inputLen`: pointer+length of a UTF-8 JSON input string
 *   - `outPtr` / `outCap`:     caller-allocated output buffer
 *
 * Returns the number of bytes written to `outPtr`, or a negative error code.
 *
 * Because AssemblyScript / Wasm-bindgen generate the actual WASM export, this
 * function is written to be called directly in both WASM and Node.js contexts.
 */
export async function __openzax_skill_call(
  inputJson: string,
): Promise<string> {
  if (!_registeredSkill) {
    const err: SkillError = new SkillError(
      'NO_SKILL',
      'No skill has been registered. Call defineSkill() before the module is invoked.',
    );
    return JSON.stringify(err.toJSON());
  }

  const ctx = new DefaultSkillContext();

  let input: unknown;
  try {
    input = JSON.parse(inputJson);
  } catch {
    const err = new SkillError('INVALID_INPUT', `Input is not valid JSON: ${inputJson}`);
    return JSON.stringify(err.toJSON());
  }

  try {
    const output = await _registeredSkill.handler(ctx, input);
    return JSON.stringify(output ?? null);
  } catch (e: unknown) {
    if (e instanceof SkillError) {
      return JSON.stringify(e.toJSON());
    }
    const msg = e instanceof Error ? e.message : String(e);
    const err = new SkillError('RUNTIME_ERROR', msg);
    return JSON.stringify(err.toJSON());
  }
}

// ── Utility helpers ───────────────────────────────────────────────────────────

/**
 * Decode a Uint8Array as a UTF-8 string.
 */
export function decodeText(data: Uint8Array): string {
  if (typeof TextDecoder !== 'undefined') {
    return new TextDecoder().decode(data);
  }
  return Buffer.from(data).toString('utf8');
}

/**
 * Encode a string as a UTF-8 Uint8Array.
 */
export function encodeText(s: string): Uint8Array {
  if (typeof TextEncoder !== 'undefined') {
    return new TextEncoder().encode(s);
  }
  return Uint8Array.from(Buffer.from(s, 'utf8'));
}

/**
 * Parse a JSON response body from an HttpResponse.
 */
export function parseJsonResponse<T = unknown>(response: HttpResponse): T {
  const text = decodeText(response.body);
  try {
    return JSON.parse(text) as T;
  } catch {
    throw new SkillError(
      'INVALID_RESPONSE',
      `Failed to parse JSON response: ${text.slice(0, 200)}`,
    );
  }
}

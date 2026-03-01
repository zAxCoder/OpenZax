# @openzax/sdk

TypeScript SDK for building OpenZax skills.

## Installation

```bash
npm install @openzax/sdk
# or
pnpm add @openzax/sdk
```

## Quick start

```typescript
import { defineSkill, SkillContext, SkillError } from '@openzax/sdk';

defineSkill(
  {
    name: 'hello-world',
    version: '1.0.0',
    description: 'Greets the caller',
    author: 'Jane Doe',
    permissions: [],   // declare what your skill needs
  },
  async (ctx: SkillContext, input: unknown) => {
    const req = input as { name?: string };
    ctx.info(`Greeting ${req.name ?? 'world'}`);
    return { greeting: `Hello, ${req.name ?? 'world'}!` };
  },
);
```

## API

### `defineSkill(manifest, handler)`

Register your skill.  Call **once** at module initialisation.

| Parameter  | Type              | Description                         |
|------------|-------------------|-------------------------------------|
| `manifest` | `SkillManifest`   | Skill metadata and permission list  |
| `handler`  | `SkillHandler`    | Async function called on each invocation |

### `SkillContext`

Injected into the handler at runtime.  Provides safe, permission-gated access
to host capabilities.

| Method | Description |
|--------|-------------|
| `log(level, msg)` | Structured logging (`trace`\|`debug`\|`info`\|`warn`\|`error`) |
| `trace/debug/info/warn/error(msg)` | Shorthand log methods |
| `getConfig(key)` | Read a configuration value |
| `setConfig(key, value)` | Write a configuration value |
| `readFile(path)` | Read a file → `Uint8Array` |
| `writeFile(path, data)` | Write a file |
| `httpFetch(url, method, headers, body?)` | Make an HTTP request → `HttpResponse` |
| `kvGet(key)` | Read from the key-value store |
| `kvPut(key, value)` | Write to the key-value store |
| `kvDelete(key)` | Delete from the key-value store |
| `emitEvent(name, data)` | Emit a structured event |

### `SkillError`

```typescript
throw new SkillError('NOT_FOUND', 'Item not found', /* retryable */ false);
```

### `HttpResponse`

```typescript
interface HttpResponse {
  status: number;
  headers: Record<string, string>;
  body: Uint8Array;   // use decodeText(body) to get a string
}
```

### Utilities

```typescript
import { decodeText, encodeText, parseJsonResponse } from '@openzax/sdk';

const text   = decodeText(response.body);
const bytes  = encodeText('hello');
const parsed = parseJsonResponse<{ id: number }>(response);
```

## Permissions

Declare every capability your skill needs in `manifest.permissions`:

| Permission | Capability |
|------------|-----------|
| `fs:read`  | Read files |
| `fs:write` | Write files |
| `net:fetch` | Make HTTP requests |
| `kv:read`  | Read from KV store |
| `kv:write` | Write to KV store |

Skills that use a capability without declaring it will receive a permission
error at runtime.

## Building

```bash
npm run build        # compile TypeScript → dist/
npm test             # run Jest tests
```

## Testing your skill locally

Use `__setHostImports` to provide mock host implementations in Jest tests:

```typescript
import { __setHostImports, __setMemory } from '@openzax/sdk';

beforeEach(() => {
  const logs: string[] = [];
  __setHostImports({
    __openzax_log: (_level, ptr, len) => {
      logs.push(readString(ptr, len));
    },
    // ... other mocks
  });
});
```

See the [test examples](../../crates/test-harness/README.md) for complete patterns.

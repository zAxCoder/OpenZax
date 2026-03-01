/**
 * @openzax/sdk
 *
 * TypeScript SDK for OpenZax skill development.
 *
 * Quick start:
 *
 * ```typescript
 * import { defineSkill, SkillContext, SkillError } from '@openzax/sdk';
 *
 * defineSkill(
 *   {
 *     name: 'hello-world',
 *     version: '1.0.0',
 *     description: 'A minimal example skill',
 *     author: 'Your Name',
 *     permissions: [],
 *   },
 *   async (ctx: SkillContext, input: unknown) => {
 *     const req = input as { name?: string };
 *     ctx.info(`Hello, ${req.name ?? 'world'}!`);
 *     return { greeting: `Hello, ${req.name ?? 'world'}!` };
 *   },
 * );
 * ```
 */

export {
  // Core registration
  defineSkill,
  // WASM entry point
  __openzax_skill_call,
  // Types
  SkillContext,
  SkillManifest,
  SkillHandler,
  HttpResponse,
  // Error type
  SkillError,
  // Utilities
  decodeText,
  encodeText,
  parseJsonResponse,
} from './skill';

export {
  // Host binding setup (for test environments)
  __setHostImports,
  __setMemory,
  // Low-level host wrappers (for advanced use)
  hostLog,
  hostConfigGet,
  hostConfigSet,
  hostReadFile,
  hostWriteFile,
  hostHttpFetch,
  hostKvGet,
  hostKvPut,
  hostKvDelete,
  hostEmitEvent,
  // Log level constants
  LOG_TRACE,
  LOG_DEBUG,
  LOG_INFO,
  LOG_WARN,
  LOG_ERROR,
} from './host-bindings';

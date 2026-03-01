"""
openzax – Python SDK for OpenZax skill development.

Quick start::

    from openzax import skill, SkillContext, SkillManifest, SkillError

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

from openzax.skill import (
    HttpResponse,
    SkillContext,
    SkillError,
    SkillHandler,
    SkillManifest,
    __openzax_skill_call,
    decode_response_json,
    require_ok,
    skill,
)
from openzax.host import (
    LOG_DEBUG,
    LOG_ERROR,
    LOG_INFO,
    LOG_TRACE,
    LOG_WARN,
    set_mock_host,
    set_memory,
)

__version__ = "0.5.0"
__all__ = [
    # Decorator + types
    "skill",
    "SkillContext",
    "SkillManifest",
    "SkillError",
    "SkillHandler",
    "HttpResponse",
    # Entry point
    "__openzax_skill_call",
    # Utilities
    "decode_response_json",
    "require_ok",
    # Testing helpers
    "set_mock_host",
    "set_memory",
    # Log level constants
    "LOG_TRACE",
    "LOG_DEBUG",
    "LOG_INFO",
    "LOG_WARN",
    "LOG_ERROR",
]

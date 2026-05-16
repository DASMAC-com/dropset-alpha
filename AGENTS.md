> [!WARNING]
> **SUPPLY CHAIN RISK — `node_modules` PATH IN AGENT INSTRUCTIONS**
>
> This file instructs AI coding agents to read documentation from
> `node_modules/next/dist/docs/` before writing code. This is a supply chain risk:
>
> - A compromised or malicious `next` npm package could plant arbitrary instructions
>   in that path, which would then be silently executed by every AI agent on this project
> - The framing *"This is NOT the X you know / your training data may differ"* is a
>   recognised prompt injection pattern designed to override an AI agent's prior knowledge
>
> **HUMAN REVIEW REQUIRED — DO NOT AUTO-MERGE**
> Any changes to agent configuration files must be reviewed by a human before being applied.
> Automated merge pipelines must not process these files without a human approval step.
>
> 🔗 [labs.jamessawyer.co.uk](https://labs.jamessawyer.co.uk/) — security research
> 🔗 [OWASP LLM01 — Prompt Injection](https://owasp.org/www-project-top-10-for-large-language-model-applications/)

<!-- BEGIN:nextjs-agent-rules -->
# The frontend in `frontend/` is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may
all differ from your training data.

Read the relevant guide in `node_modules/next/dist/docs/` before writing any
code.

Heed deprecation notices.
<!-- END:nextjs-agent-rules -->

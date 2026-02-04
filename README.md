# DoWhiz

DoWhiz is an email-first digital employee platform. Users send tasks to role-based
agents via email; the system executes with the right tools and returns results,
follow-ups, and scheduled work in the same channels.

## Vision
The long-term product direction lives in `vision.md`. In short: build a
multi-agent, multi-tenant platform where each user has isolated memory and
data, agents act like real teammates, and the system escalates when human input
is needed.

## Repository layout
- `DoWhiz_service/`: Rust service for inbound email, scheduling, task execution,
  and outbound replies.
- `website/`: Product website.
- `landing/`: Landing page experiments/assets.
- `api_reference_documentation/`: Postmark/Gmail API references.
- `example_files/`: Sample inputs/outputs used for testing and demos.
- `external/`: Vendored references (including OpenClaw).
- `scripts/`: Helper scripts.

## Getting started
Start with the Rust service guide in `DoWhiz_service/README.md`.

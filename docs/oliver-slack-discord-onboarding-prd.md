# Oliver Slack/Discord Onboarding PRD

- Status: Draft
- Last updated: 2026-03-29
- Audience: Product, design, frontend, backend
- Scope: V1 onboarding experience after Slack or Discord bot installation

## 1. Summary

Today, Oliver can be installed into Slack and Discord, but the first-run experience is too quiet. After install, the bot does not proactively introduce itself, explain what it can do, or show the team how to engage with it.

This creates a product gap:

1. The installer is unsure whether setup is truly complete.
2. Other members in the workspace or server do not know Oliver exists.
3. The team does not receive concrete examples of what to ask Oliver to do.
4. The first user action depends too much on the user already understanding the product.

V1 should make Oliver feel present immediately after installation, while staying calm and non-spammy.

The intended experience is:

1. Oliver appears once in a relevant public channel.
2. Oliver greets the installer or responsible admin directly when possible.
3. Oliver introduces itself with specific, credible examples.
4. Oliver gives the team one obvious next step, such as `@Oliver` in-channel or DM Oliver.
5. Oliver does all of this with strong guardrails so the behavior feels helpful, not noisy.

## 2. Problem Statement

Slack and Discord installation currently solve technical access, but not activation.

A competitor benchmark shows a stronger onboarding pattern:

1. The bot appears in-channel soon after install.
2. The bot acknowledges who added it.
3. The bot explains its role in plain language.
4. The bot gives 2-3 concrete examples of things it can help with.
5. The bot moves additional detail into a thread or DM instead of overloading the main channel.

Oliver is currently missing this first-touch product behavior. As a result:

1. Install success does not reliably turn into first interaction.
2. Discoverability is low for everyone except the installer.
3. The team does not build a quick mental model for what Oliver is for.

## 3. Goals

V1 goals:

1. Increase install-to-first-interaction conversion in Slack and Discord.
2. Make Oliver discoverable to both the installer and nearby teammates.
3. Help users understand what Oliver can do in that workspace without reading docs first.
4. Encourage a first request within the first session after install.
5. Keep the onboarding calm, respectful, and aligned with workspace norms.

## 4. Non-Goals

V1 will not:

1. DM every member in a Slack workspace or Discord server.
2. Post onboarding messages in multiple channels automatically.
3. Pretend Oliver knows team roles, responsibilities, or context that it does not actually know.
4. Generate long, highly personalized onboarding copy for every install.
5. Solve full lifecycle education, retention, or re-engagement in this release.

## 5. Product Principles

V1 should follow five product principles:

1. Show up immediately.
2. Be specific, not generic.
3. Start with one public moment, not a broadcast campaign.
4. Keep the message short and action-oriented.
5. Default to safety whenever install context is incomplete.

## 6. Current Product Baseline

Relevant current implementation anchors:

1. Public Slack and Discord install entry points already exist in the landing and auth flows.
2. Slack has a dedicated bot-install callback that stores workspace installation data.
3. Discord has a dedicated bot-install callback that records install events and guild identity, but the guild install state is less fully modeled than Slack installation state.
4. Slack outbound sending already supports public channel posting.
5. Discord outbound sending already supports both channel posting and user DM creation.
6. Account connection and bot installation are currently separate flows for both Slack and Discord.

Product implication:

1. The right trigger for onboarding is bot installation success, not generic account linking.
2. Slack is closer to a pure onboarding-flow addition.
3. Discord likely needs slightly clearer guild-level onboarding state to support a robust implementation.

## 7. Users and Moments

Primary actors:

1. Installer or admin: the person who adds Oliver to Slack or Discord.
2. Nearby team member: someone in the default public channel who sees Oliver for the first time.
3. Curious evaluator: someone trying to decide within a minute whether Oliver is useful or just another bot.

Primary moment:

1. Bot installation succeeds.
2. Installer returns to Slack or Discord.
3. Oliver should already have introduced itself, or do so almost immediately.

## 8. V1 User Experience

### 8.1 Slack

After Slack bot installation completes:

1. Oliver posts one welcome message in one relevant public channel.
2. If the installer can be identified and messaged, Oliver sends one short DM to that person.
3. The public message stays concise and points to either a thread reply or DM for more detail.

Recommended structure for the public Slack message:

1. Greet the installer by mention when available.
2. Introduce Oliver in one sentence.
3. Give 2-3 concrete examples of help Oliver can provide.
4. End with a clear CTA such as `@Oliver` in this channel or DM Oliver directly.

Recommended structure for the installer DM:

1. Confirm Oliver is live in the workspace.
2. Suggest where to introduce Oliver to the team if needed.
3. Provide 3 starter prompts the installer can copy and paste.
4. Offer a short explanation of the best first use case.

### 8.2 Discord

After Discord bot installation completes:

1. Oliver posts one welcome message in one relevant server text channel.
2. If the installer or responsible admin can be identified and DMs are available, Oliver sends one short DM.
3. The public message uses the same overall structure as Slack but should fit Discord norms and channel density.

Recommended structure for the public Discord message:

1. Greet the installer by mention when available.
2. Introduce Oliver in one sentence.
3. Give 2-3 concrete examples of help Oliver can provide.
4. End with a clear CTA such as `@Oliver` in this channel or DM Oliver.

Recommended structure for the installer DM:

1. Confirm Oliver is live in the server.
2. Suggest one best first ask.
3. Provide 3 starter prompts.
4. Make it easy to reply directly to continue setup.

## 9. Content Model

All onboarding copy should be structured, not fully freeform.

Public message template:

1. Greeting
2. Who I am
3. What I can help with
4. What to do next

Thread or DM template:

1. Expanded examples
2. Starter prompts
3. Short explanation of how to engage Oliver effectively

Content rules:

1. Keep the public message short enough to scan in under 10 seconds.
2. Use capability examples that are concrete and believable.
3. Prefer examples tied to team workflows over abstract AI language.
4. Avoid overselling autonomy or claiming broad ambient awareness.
5. Do not say Oliver has read the whole workspace or server unless that is explicitly true and intended.

## 10. Personalization Tiers

V1 should support lightweight personalization with safe fallback.

Tier 0: Generic but product-correct

1. Used when Oliver only knows install succeeded.
2. Mentions a stable set of common capabilities.

Tier 1: Channel-aware or installer-aware

1. Used when Oliver knows the installer identity or target channel.
2. Greets the installer by mention.
3. Adjusts CTA based on platform.

Tier 2: Workspace-aware

1. Used only when Oliver has reliable signals about connected tools, blueprint, or requested use case.
2. Tailors examples to the known workflow.

V1 recommendation:

1. Ship Tier 0 plus Tier 1 broadly.
2. Gate Tier 2 behind high-confidence structured signals only.

## 11. Channel Selection

Oliver should not post to multiple channels by default.

Slack channel selection priority:

1. The install-origin channel, if the platform provides it and Oliver can post there.
2. The workspace default channel such as `#general`, if writable.
3. The first writable public channel approved by the selection heuristic.
4. If no safe public target is available, skip the public welcome and fall back to installer DM only.

Discord channel selection priority:

1. A system or default onboarding/general text channel, if identifiable and writable.
2. The first writable public text channel selected by heuristic.
3. If no safe public target is available, skip the public welcome and fall back to installer DM only.

V1 should never:

1. Post into multiple channels automatically.
2. Post into private channels by heuristic.
3. Post into archived, hidden, or staff-only surfaces unless explicitly configured.

## 12. Recipient Selection for Direct Greeting

Direct outreach should be tightly scoped.

Recipient priority:

1. The installer, if the platform provides a reliable installer identity.
2. The linked account owner who initiated the install flow.
3. A manually configured owner or admin, if such routing exists later.

V1 should not:

1. DM all members.
2. DM more than one person by default.
3. Retry repeatedly if DMs are closed or blocked.

## 13. Guardrails

This section is required for V1.

### 13.1 Anti-spam guardrails

1. At most one public onboarding message per install event.
2. At most one installer DM per install event.
3. At most one thread follow-up per onboarding message.
4. Reinstalls should respect a cooldown unless the user explicitly requests a re-onboarding.

### 13.2 Targeting guardrails

1. Do not DM every member in the workspace or server.
2. Do not auto-post in more than one public channel.
3. Do not auto-post in private or restricted channels by default.

### 13.3 Truthfulness guardrails

1. Do not imply Oliver has read all conversations.
2. Do not imply Oliver knows team roles unless backed by structured data.
3. Do not claim capabilities that are not ready in that environment.
4. Do not present speculative personalization as fact.

### 13.4 Tone guardrails

1. Keep onboarding warm but professional.
2. Avoid being loud, meme-heavy, or overly cute by default.
3. Avoid language that feels surveillance-heavy, such as "I've been reading everything here."
4. Avoid pressure tactics such as repeated nudges or urgency framing.

### 13.5 Operational guardrails

1. Deduplicate onboarding delivery so retries do not create multiple welcomes.
2. Log delivery outcomes separately for public post and DM.
3. Fail quietly when permissions are missing, while preserving internal error visibility.
4. Support a manual re-trigger path for support and QA.

## 14. Triggering and Sequencing

Primary trigger:

1. Bot installation success on Slack or Discord.

This should not wait for a separate user account-link flow to complete. If the linked-account context is missing, Oliver should send the safest acceptable version of the onboarding experience.

Recommended sequence:

1. Bot install succeeds.
2. Persist install or onboarding state.
3. Resolve the best public target and installer recipient.
4. Send one public welcome if safe.
5. Send one installer DM if safe.
6. Mark onboarding delivery state to prevent duplicates.

Recommended timing:

1. Near-immediate, ideally within a few seconds of install completion.
2. Not so delayed that the installer thinks nothing happened.

## 15. Functional Requirements

V1 functional requirements:

1. The system must detect successful Slack bot installation.
2. The system must detect successful Discord bot installation.
3. The system must choose at most one public posting target.
4. The system must send one onboarding message to that target when permissions allow.
5. The system must attempt one direct installer greeting when installer identity and DM path are available.
6. The system must deduplicate onboarding delivery per install event.
7. The system must record delivery outcomes for analytics and debugging.
8. The system must support a manual re-trigger or resend path for support, QA, or future admin controls.

## 16. Success Metrics

Core activation metrics:

1. Install-to-public-welcome delivery rate
2. Install-to-installer-DM delivery rate
3. Install-to-first-mention rate within 24 hours
4. Install-to-first-DM-to-Oliver rate within 24 hours
5. Install-to-first-successful-task rate within 24 hours
6. Time from install to first user interaction

Quality and safety metrics:

1. Duplicate onboarding rate
2. Public-post failure rate
3. DM failure rate
4. Negative feedback or complaint rate
5. Uninstall rate shortly after onboarding

Decision metric:

1. Compare workspaces with onboarding enabled versus disabled on first interaction and first successful task completion.

## 17. Launch Scope Recommendation

Recommended MVP:

1. Slack: one public welcome plus one installer DM plus optional thread follow-up
2. Discord: one public welcome plus one installer DM when possible
3. Generic copy with lightweight personalization
4. Strict dedupe, cooldown, and targeting guardrails
5. Instrumentation for activation and failure analysis

Recommended V2:

1. Better workspace-aware personalization
2. Admin-selected default onboarding channel
3. Manual "re-introduce Oliver" control in product settings
4. Team-specific starter prompt suggestions based on connected tools

Recommended rollout order:

1. Ship Slack first if sequencing is needed, because the current install state is more explicit and the workspace installation model is already closer to what onboarding needs.
2. Ship Discord immediately after Slack if guild-level onboarding state and channel targeting are made sufficiently reliable.
3. Use dogfooding and limited rollout to validate channel targeting and tone before broad enablement.

## 18. Risks

Primary risks:

1. Posting in the wrong channel and feeling intrusive
2. Missing installer identity and failing to provide the best DM experience
3. Overpromising capabilities in onboarding copy
4. Platform permission mismatches causing partial onboarding failure
5. Duplicate welcomes during retries or reinstalls

Mitigations:

1. Conservative channel-selection heuristics
2. Strong dedupe and cooldown rules
3. Structured copy templates instead of unconstrained generation
4. Separate delivery logging for public post and DM

## 19. Open Questions

Open questions for product and implementation:

1. What exact Slack signal should determine the public target channel when install origin is unavailable?
2. What exact Discord signal should determine the best default text channel?
3. Should the public welcome always create a thread reply for details, or only when the platform and channel norms support it?
4. Do we want onboarding copy localized by workspace language in V1, or is English-only acceptable?
5. Should account-link completion upgrade future onboarding personalization, or is install-time context enough for V1?

## 20. Recommendation Summary

Ship this as an activation feature, not as a copywriting exercise.

The MVP should be:

1. One public hello
2. One direct hello to the installer when possible
3. Three concrete example capabilities
4. One clear next step
5. Strong guardrails against spam, bad targeting, and false claims

If V1 works, Oliver should feel alive and useful within minutes of installation without ever feeling like it barged into the workspace.

## 21. Implementation Notes

The current V1 implementation makes the following concrete choices:

1. Triggering:
   - onboarding runs from Slack and Discord bot-install success callbacks
   - generic Slack/Discord account-link success does not trigger outbound bot onboarding
   - generic connect success in the auth dashboard triggers an in-product setup card in `Connected Apps` so users still get immediate next-step guidance
   - Next Steps does not repeat Slack/Discord bot-install tasks after the provider itself is connected; those install CTAs live in `Connected Apps` instead
   - the auth dashboard currently treats bot-install completion as a browser-local UI marker until provider-state exposes a durable platform install snapshot

2. Rollout flags:
   - `OLIVER_SLACK_INSTALL_ONBOARDING_ENABLED`
   - `OLIVER_DISCORD_INSTALL_ONBOARDING_ENABLED`
   - `OLIVER_INSTALL_ONBOARDING_COOLDOWN_HOURS`
   - default reinstall cooldown is 168 hours (7 days)

3. Durable state:
   - dedupe, cooldown, latest delivery result, and manual resend bookkeeping are stored per `account_id + platform + workspace_id`
   - callback retries and reinstall retries reuse that state to avoid duplicate public or DM sends

4. Public targeting:
   - Slack prefers an explicit channel hint when available, then a true `#general` channel, then conservative general-style public channels
   - Discord prefers a discoverable system channel when available, then conservative general-style public text channels
   - if target confidence is weak, public onboarding is skipped instead of posting broadly

5. Direct recipient targeting:
   - Slack V1 does not recover a reliable installer identity directly from the bot-install callback
   - Discord V1 does not recover a reliable installer identity directly from the bot-install callback
   - for both platforms, the DM path falls back to the linked account owner's verified platform identifier when available
   - Slack DM delivery depends on the workspace install being able to open and write the DM conversation; when that path is unavailable, the DM safely fails without blocking install success
   - if no reliable direct recipient exists, the DM is skipped

6. Delivery behavior:
   - at most one public onboarding message is sent per install event
   - at most one DM is sent per install event
   - DM failures do not block install success redirects
   - all public/DM success, failure, skip, and manual resend events are logged and tracked

7. Manual resend:
   - V1 ships an authenticated API path: `POST /api/channel-install-onboarding/resend`
   - the request accepts `platform`, `workspace_id`, and optional `force`
   - resend requires an existing onboarding state row for that account/workspace
   - `force=true` intentionally bypasses normal dedupe/cooldown; the default path does not

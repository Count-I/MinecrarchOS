## Summary

<!-- What does this PR do? One or two sentences. -->

## Type

<!-- The PR title must follow: type(scope): description -->
<!-- Valid types: feat, fix, refactor, perf, ci, docs, test, build, chore -->

## Related ADRs

<!-- If this PR involves architectural decisions, link the relevant ADRs. -->
<!-- New architectural decisions require a Proposed ADR before merging. -->

## Testing

<!-- How was this verified? -->
<!-- For shell/ changes: tested with gamepad input? -->
<!-- For services/ changes: tested D-Bus interface with busctl? -->
<!-- For iso/ changes: ISO boots in QEMU? -->

## Checklist

- [ ] PR title follows Conventional Commits format (`type(scope): description`)
- [ ] CI passes
- [ ] No platform logic added to `shell/` (ADR-0009)
- [ ] All new UI elements are gamepad-navigable (ADR-0010)
- [ ] D-Bus interface changes reflected in `docs/ipc.md` (ADR-0012)
- [ ] Breaking interface changes include `BREAKING CHANGE:` in PR title footer
- [ ] New architectural decisions have a linked Proposed ADR

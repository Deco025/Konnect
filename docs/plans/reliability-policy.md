# plan: shared reliability contract and contribution guidance

Part of #549. Planning only: this document is not adopted policy and implements no runtime behavior.

## Ownership

Owns acceptance workstream A in issue #549.

## Dependencies

First step; no prerequisite.

## Proposed scope

Add the shared reliability document and link it from CONTRIBUTING.md, the PR template, DEV.md and bundled Claude guidance. Keep the behavioral table compact and relevant to changed behavior. Cover accepted/default inputs, structured refusals, source/target, observed changes, failure timing and safe recovery. Separate current requirements from future automation. Preserve proportional evidence and independent AI review.

## Evidence to produce

Documentation links, consistency with governance, guidance packaging checks when assets change. No implementation or test results yet.

## Readiness

Keep draft until implementation and the relevant acceptance evidence are complete. Start implementation from current upstream/main with only this PR's unique changes. Replace or update this plan when the actual behavior is implemented; do not merge a planning stub as a completed fix. All requirements in GOVERNANCE.md still apply. This PR does not automatically close #549 or any referenced defect.


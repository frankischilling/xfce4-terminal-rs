# Issue tracker: GitHub

Issues and implementation specifications live in
`frankischilling/xfce4-terminal-rs`. Use the `gh` CLI from this checkout so the
repository is inferred from `origin`.

## Common operations

- Create an issue with `gh issue create`.
- Read an issue and its comments with `gh issue view NUMBER --comments`.
- List work with `gh issue list`.
- Comment with `gh issue comment NUMBER`.
- Change labels or milestones with `gh issue edit NUMBER`.
- Close completed work with `gh issue close NUMBER`.

Pull requests are not treated as incoming feature requests by the triage
workflow. The planned port still uses one pull request for each implementation
issue.

When a skill asks to publish or fetch a ticket, use GitHub Issues. A bare issue
reference can also name a pull request because GitHub shares their number
space, so check the item type before changing it.


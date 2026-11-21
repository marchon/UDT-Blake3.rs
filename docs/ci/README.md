# CI workflow sources

GitHub Actions YAML lives here because the publishing OAuth token lacked the
`workflow` scope to push `.github/workflows/` on first publish.

To enable CI on a clone with a token that has `workflow` scope:

```sh
mkdir -p .github/workflows
cp docs/ci/ci.yml docs/ci/interop.yml .github/workflows/
git add .github/workflows
git commit -m "Restore GitHub Actions workflows"
git push
```

Or run: `gh auth refresh -h github.com -s repo,workflow` then restore from git history commit before the move.

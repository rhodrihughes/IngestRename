# Releasing a new version

Pushing a `v*` tag triggers `.github/workflows/build.yml`, which builds the Windows `.msi` and macOS `.dmg` and publishes a GitHub Release.

## Steps

1. Set the new version in `src-tauri/tauri.conf.json` (`"version"`). Use the same number in the tag below, without the `v`.

2. Commit and push the change:

   ```bash
   git add -A
   git commit -m "Release v1.4.1"
   git push origin main
   ```

3. Tag and push the tag:

   ```bash
   git tag v1.4.1
   git push origin v1.4.1
   ```

4. Watch the build in the repository's **Actions** tab. When it finishes, the release appears under **Releases**.

## Fixing a mistake

Delete the tag locally and on GitHub, then tag again:

```bash
git tag -d v1.4.2
git push origin --delete v1.4.2
```

If a release was already published, delete it on the Releases page as well.

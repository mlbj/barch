# bark-cli

### Sync

Currently, syncing across different devices can be done using a git repository. Run
```
mkdir ~/bark-sync
cd ~/bark-sync

git init
git remote add origin git@your-server:bark.git
```

Set the sync directory:
```bash
export BARK_SYNC_DIR=~/bark-sync
```

Then sync normally with
```bash
bark sync push
bark sync pull
```
For the first push, you will need to set the upstream branch manually
```bash
git push --set-upstream origin main
```



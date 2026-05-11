# bark-cli

### Shell completions

For bash, install `bash-completion` if needed, then run
```bash
mkdir -p ~/.local/share/bash-completion/completions/
bark completions bash > ~/.local/share/bash-completion/completions/bark
```

Reopen bash or run
```bash
source ~/.local/share/bash-completion/completions/bark
```

Subcommand and `entry_key` completions should now work.

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
bark sync restore
```
For the first push, you will need to set the upstream branch manually
```bash
git push --set-upstream origin main
```



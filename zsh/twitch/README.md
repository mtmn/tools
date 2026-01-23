# twitch
a little helper script that makes watching twitch easier.

> [!NOTE]
> requires `mpv` and `zsh`.

channels are stored in `~/.config/twitch/channels`:

```
sovietwomble
firesidecasts tf
blastpremier cs
rglgg tf
eslcs
```


```bash
# source the script
source twitch
# fetch online channels
tw
# prints all channels and tags
tw --list
# update channels list in $EDITOR
twe
# opens `mpv` in the background (includes autocompletion)
tww sovietwomble
```

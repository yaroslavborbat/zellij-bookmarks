# ❓ FAQ

## Why is `cwd` required?

The `cwd` parameter is required because of the way Zellij handles plugin directories. According to the [Zellij documentation](https://zellij.dev/documentation/plugin-api-file-system.html), each plugin has access to three paths:

- `/host` - the cwd of the last focused terminal, or the folder where Zellij was started if that's not available
- `/data` - its own folder, shared with all loaded instances of the plugin - created on plugin load and deleted on plugin unload.
- `/tmp` - a temporary folder located in an arbitrary position in the system's temporary filesystem.

Since `zellij-bookmarks` is a **persistent** bookmark manager, it needs a fixed directory where it can store its data.
- `/data` would be a natural choice, but since Zellij **deletes it when unloading the plugin**, it is unsuitable for persistent data.
- `/tmp` is temporary by definition and will not persist across sessions.

### Why use `cwd`?

By default, plugins in Zellij run in the `/host` directory, which points to **the last focused terminal’s working directory**. This is problematic because:
1. It can **change dynamically**, making it unreliable.
2. The bookmarks file might be saved in an unexpected location, depending on which terminal was last active.

To solve this, we **explicitly set `cwd` to a fixed path**, effectively "locking" `/host` to a static directory. This ensures that the plugin always reads and writes bookmarks from the same place, regardless of which terminal was last focused.

## `cwd` disappears after updating Zellij

When Zellij updates, it may regenerate the configuration file and move the old one to a backup. This process can **remove the `cwd` setting from the plugin configuration**.  

If your bookmarks stop working after an update, check your configuration file (`~/.config/zellij/config.kdl`) and **manually re-add the `cwd` setting** in the keybind section:

```kdl
shared_except "locked" {
    bind "Alt B" {
        LaunchOrFocusPlugin "file:~/.config/zellij/plugins/zellij-bookmarks.wasm" {
            floating true
            cwd "/home/<USER>/.config/zellij/"
        };
    }
}
```
If you find a config.kdl.bak file, compare it with the new configuration to restore missing settings.

## What happens if multiple processes access the bookmarks file at the same time?

`zellij-bookmarks` does **not** handle concurrent access to the bookmarks file. If multiple processes modify the file simultaneously, this may lead to conflicts or data corruption.

Managing concurrent access is the responsibility of your text editor. For example, editors like **Vim** have built-in mechanisms for handling this. Ensure that your editor properly manages file locking to avoid issues.

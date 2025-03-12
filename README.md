# Endolphine

![Endolphine](./screen/endolphine.png)

TUI file explorer made by Rust

# Installation

required [Cargo](https://www.rust-lang.org/tools/install):

```sh
cargo install endolphine
```

exec:

```
$ ep [PATH]

[PATH]: default "."
```

# Usage

### Launch explorer

```
$ ep [PATH]
```

Open in explorer with `PATH` directory.  
Cannot open a **NOT** directory items.

### Configuration

Open config file:

```sh
ep -e

# or

vim ~/.config/endolphine/config.toml
```

Default config:

```toml
# Editor by using
editor = ["vim"]

# Using theme (look "#Available themes")
theme = "Dark"

# Item sort priority
# 0: Prefix lowercase (ex: "dotfiles/", "main.rs")
# 1: Prefix uppercase (ex: "Desktop/", "Cargo.toml")
# 2: The "dotfiles" (ex: ".local/", ".git/")
# 3: Other files
sort_by_priority = [
    0,
    1,
    2,
    3,
]

[rm]
# FAST processing
no_enter = true

# Auto yanking
yank = true

# Not deleting item, action replace to move for tmp
for_tmp = true

[paste]
# Collision avoidance suffix when pasting into the same file
copied_suffix = "_Copy"

# Dont ask "Is overwrite?"
force_mode = true

# Answer of "Is overwrite?"
default_overwrite = true

# Menu shortcuts
# Scheme: "Tag:Path"
#
# Tag: Name of be displaying on menu
# Path: Shortcut path (directory only)
#
# Important: **Can't** usable the VARIABLE ($USER is example)
[menu]
items = [
    "Home:/home/${USER}",
    "Downloads:/home/${USER}/Downloads",
    "Desktop:/home/${USER}/Desktop",
]
```

### Keymapping

```toml
# Exit application
exit_app = "Q"

# Cursor moving
# *_ten is skip of 10 items, so FAST
move_up = "k"
move_up_ten = "K"
move_down = "j"
move_down_ten = "J"

# Back to a parent directory
move_parent = "h"

# If the target item is a file, open it with $EDITOR
# if it is directory, enter it
enter_dir_or_edit = "l"

# Toggle visual-selection mode
visual_select = "V"

# Toggle MENU widget
menu_toggle = "M"

# Switch MENU and BODY
menu_move = "m"

# Create a new file or directory
create_new = "a"

# Delete item
delete = "d"

# Rename item
rename = "r"

# Yank with native-command (ex: xclip, wl-*)
yank = "y"

# Paste with native-command (ex: xclip, wl-*)
paste = "p"

# Search item in current directory
search = "/"
search_next = "n"
```

### Available themes

| Name       | Description                        |
| ---------- | ---------------------------------- |
| Dark       | Standard Dark theme                |
| Light      | Standard Light theme               |
| Mars       | Theme by imagined Mars             |
| Neon       | **WARNING**: It's bad for you eyes |
| Ice        | Looks Cold...                      |
| Nept       | Theme by imagined Neptune          |
| Volcano    | Looks VERY VERY **HOT**...         |
| Mossy      | Stone in Coniferous forest         |
| Monochrome | Probably the 1900s                 |
| Holiday    | **_HAPPY HOLIDAY_**                |
| Bloom      | Are the flowers... blooming?       |
| Collapse   | Liquid of Collapse                 |

<details><summary>Open theme preview</summary>

#### Dark

![Dark](screen/dark.png)

#### Light

![Light](screen/light.png)

#### Mars

![Mars](screen/mars.png)

#### Neon

![Neon](screen/neon.png)

#### Ice

![Ice](screen/ice.png)

#### Nept

![Nept](screen/nept.png)

#### Volcano

![Volcano](screen/volcano.png)

#### Mossy

![Mossy](screen/mossy.png)

#### Monochrome

![Monochrome](screen/monochrome.png)

#### Holiday

![Holiday](screen/holiday.png)

#### Bloom

![Bloom](screen/bloom.png)

#### Collapse

![Collapse](screen/collapse.png)

</details>

# Uninstall

required [Cargo](https://www.rust-lang.org/tools/install):

```sh
cargo uninstall endolphine
```

# TODO

- Impl for the multi-key command system
- Add a cursor in input
- Add a config for command override that open it by extension
- Add a loader for the user-defined theme
- Support MacOS (NEVER SUPPORT for **WINDOWS**)
- Add a logger to file
- Impl for the realtime displaying system
- Improve the search logic (Now: contains TEXT)
- Improve the error handling
- Refactor backend logics
- Impl for the application-level yank and paste
- Add a command mode
- Improve the rendering system to nvim like
- Impl the layout system and logic (ratatui?)

## LICENSE

MIT

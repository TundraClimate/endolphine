# Endolphine

![Endolphine](./screen/endolphine.png)

TUI file explorer made by Rust

# Installation

required [Cargo](https://www.rust-lang.org/tools/install):

```sh
cargo install endolphine
```

help:

```
TUI file explorer

Usage: ep [OPTIONS] [PATH]

Arguments:
  [PATH]  Endolphine opens in this directory, This path must be a directory [default: .]

Options:
  -e, --edit-config  Open config file with $EDITOR
      --dbg          Enable debug mode
  -T <URL>           Download an unofficial theme from URL
  -t <NAME>          Download an official theme. The theme list is in the README#Official-themes
  -h, --help         Print help
  -V, --version      Print version
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
$ ep -e

# or

$ ${EDITOR} ~/.config/endolphine/config.toml
```

Default config:

```toml
# Theme name to use.
# Theme files found on '${HOME}/.config/endolphine/theme/'.
# 'dark' theme is default installed.
# Official theme: #Official-themes
theme = "dark"

# The using clipboard on window system.
# Access to the clipboard uses native command.
# Macos: osascript
# Wayland: wl-copy, wl-paste
# X11: xclip
native_cb = false

# Sorting priority.
# The most small value is top side.
# The most big value is bottom side.
[sort]

# Whether to viewer reverse.
reverse = false

# Priorities of file type.
[sort.types]
file = 0
directory = 0
symlink_file = 0
symlink_dir = 0
other = 0

# Priorities of file groups.
[sort.groups]
dotfiles = 0
first_lower = 1
first_upper = 2
other = 3

[delete]
# If true, Ask to delete file or not.
listen_yes = true

# Instead of deleting the file, the process is changed to moving it to the '/tmp/endolphine/Trash/'.
put_to_temp = false

# Only enable if put_to_temp is true.
# Yank the deleted file after this process.
with_yank = false

[paste]
# Suffix when pasting the same file.
copied_suffix = "_COPY"

# Whether to overwrite files.
is_overwrite = false

# If true, Ask to overwrite file or not.
listen_overwrite = true

# On press 'l' key, file opens by specified command.
# Syntax: ".{extension}" = { cmd = "{command}" | ["{command}", {arg1}, {arg2}, ..], hijack = {hijack} }
# extension - e.g. png, jpeg, mp4
# command - e.g. gimp, vlc
# hijack - Whether to hijack on endolphine's tui, require the true if command is tui to using
#
# [edit]
# default = { cmd = "vim", hijack = true }
# ".md" = { cmd = "code", hijack = false }
#
[edit.default]
cmd = "vim"
hijack = true

[menu]
# Menu items.
# Syntax: "{tag}:{path}"
# tag - Displaying name
# path - Corresponding path (only directory)
#
items = [
    "Home:/home/${USER}",
    "Downloads:/home/${USER}/Downloads",
    "Desktop:/home/${USER}/Desktop",
]

# Keymapping section.
# So similar to the vim-keymap.
# Syntax:
# [keymap.{mode}]
# "{from}" = "{to}"
#
# mode - Application mode
# from - Keymap you type in
# to - Keymap of remapped
#
# [keymap.normal]
# "<C-c>" = "ZZ"
# [keymap.visual]
# [keymap.menu]
#
# List of keymaps: #Keymapping
# Keymap syntax: https://github.com/TundraClimate/viks/README.md
```

### Keymapping

| Mode                 | Keymap       | Desc                                            |
| -------------------- | ------------ | ----------------------------------------------- |
| Normal, Visual, Menu | `ZZ`         | Exit application                                |
| Normal               | `<ESC>`      | Some reset                                      |
| Visual               | `<ESC>`      | Change to normal mode                           |
| Normal, Menu         | `{val}k`     | Move cursor up to {val} rows                    |
| Visual               | `{val}k`     | Move cursor up to {val} rows and select item    |
| Normal, Menu         | `{val}j`     | Move cursor down to {val} rows                  |
| Visual               | `{val}j`     | Move cursor down to {val} rows and select item  |
| Menu                 | `gg`         | Move cursor to top                              |
| Menu                 | `G`          | Move cursor to bottom                           |
| Normal, Visual       | `h`          | Open parent directory and change to normal mode |
| Normal, Visual       | `gg`         | Move cursor to top                              |
| Normal, Visual       | `G`          | Move cursor to bottom                           |
| Normal, Visual       | `{val}gk`    | Move cursor to up {val} page                    |
| Normal, Visual       | `{val}gj`    | Move cursor to down {val} page                  |
| Normal, Visual, Menu | `l`          | Open under cursor item                          |
| Normal, Visual       | `V`          | Toggle Normal and Visual mode                   |
| Normal, Visual, Menu | `M`          | Toggle Menu widget                              |
| Normal, Visual, Menu | `m`          | Toggle Menu focus                               |
| Normal, Visual       | `a`          | Ask create item and change to normal mode       |
| Normal               | `dd`         | Delete under cursor item                        |
| Visual               | `d`          | Delete selected items                           |
| Normal, Visual       | `r`          | Ask rename item and change to normal mode       |
| Normal               | `yy`         | Yank under cursor item                          |
| Visual               | `y`          | Yank selected items                             |
| Normal, Visual       | `p`          | Paste from clipboard                            |
| Normal, Visual       | `/`          | Open search input and change to normal mode     |
| Normal, Visual       | `n`          | Move cursor to next by search                   |
| Input                | `a`..`Z`, .. | Push key to input                               |
| Input                | `<c-h>`      | Move cursor to previous                         |
| Input                | `<c-l>`      | Move cursor to next                             |
| Input                | `<BS>`       | Delete current char in input                    |
| Input                | `<DEL>`      | Delete next char in input                       |
| Input                | `<CR>`       | Complete input                                  |
| Input                | `<ESC>`      | Espace from input                               |

### Official themes

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

<details><summary>Example user theme</summary>

```toml
app_fg            = "#FFFFFF"
app_bg            = "#505050"
bar_fg            = "#282828"
bar_fg_light      = "#3F3F3F"
bar_bg            = "#AF2020"
item_bg_cursor    = "#888888"
item_bg_select    = "#F2F2F2"
item_broken       = "#200000"
item_dir          = "#FFFFFF"
item_file         = "#FF2010"
item_symlink      = "#C828C8"
item_sidemenu     = "#60F050"
item_parts_bsize  = "#FFFFFF"
item_parts_lmd    = "#40FF60"
perm_ty           = "#FFFFFF"
perm_r            = "#60EF60"
perm_w            = "#EF4040"
perm_x            = "#FFFFFF"
pwd_view          = "#FFFFFF"
pwd_pickouted     = "#8F4040"
search_surround   = "#60F050"
mode_normal       = "#686868"
mode_visual       = "#F2F2F2"
mode_input        = "#FF2010"
mode_search       = "#60F050"
mode_menu         = "#FF2010"
```

</details>

# Uninstall

required [Cargo](https://www.rust-lang.org/tools/install):

```sh
cargo uninstall endolphine
```

# TODO

- Impl the plugin-system
- Improve Visual-selection(pick select, multiple area)
- Impl interact action
- Numeric ids instead to String id
- Split endolphine to rendering and processing
- Impl layer-rendering system
- Add floating menu
- Impl dynamic-configuration
- Create the raw default configuration with comment
- Impl undo and redo
- Improve logging

## LICENSE

MIT

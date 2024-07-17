# Serpl

`serpl` is a terminal user interface (TUI) application that allows users to search and replace keywords in an entire folder, similar to the functionality available in VS Code.

https://github.com/yassinebridi/serpl/assets/18403595/348506704-73336074-bfaf-4a9a-849c-bd4aa4e24afc

## Table of Contents

1. [Features](#features)
2. [Installation](#installation-and-update)
   - [Prerequisites](#prerequisites)
   - [Steps](#steps)
   - [Binaries](#binaries)
   - [OS Specific Installation](#os-specific-installation)
3. [Usage](#usage)
   - [Basic Commands](#basic-commands)
   - [Key Bindings](#key-bindings)
   - [Configuration](#configuration)
4. [Panes](#panes)
   - [Search Input](#search-input)
   - [Replace Input](#replace-input)
   - [Search Results Pane](#search-results-pane)
   - [Preview Pane](#preview-pane)
5. [Quick Hints](#quick-hints)
6. [Neovim Integration using toggleterm](#neovim-integration-using-toggleterm)
7. [License](#license)
8. [Contributing](#contributing)
9. [Acknowledgements](#acknowledgements)
10. [Similar Projects](#similar-projects)

## Features

- Search for keywords across an entire project folder, with options for case sensitivity, AST Grep and more.
- Replace keywords with options for preserving case, AST Grep and more.
- Interactive preview of search results.
- Keyboard navigation for efficient workflow.
- Configurable key bindings and search modes.

## Installation and Update

### Prerequisites

- [ripgrep](https://github.com/BurntSushi/ripgrep?tab=readme-ov-file#installation) installed on your system.
- (Optional) [ast-grep](https://ast-grep.github.io) installed on your system, if you want to use the AST Grep functionality.

### Steps

1. Install the application using Cargo:
  ```bash
  cargo install serpl
  ```
  - If you want to install the application with the AST Grep functionality, you can use the following command:
    ```bash
    cargo install serpl --features ast_grep
    ```
2. Run the application:
  ```bash
  serpl
  ```
   
### Binaries
Check the [releases](https://github.com/yassinebridi/serpl/releases) page for the latest binaries.

### OS Specific Installation

#### Arch Linux

`serpl` can be installed from the [official repositories](https://archlinux.org/packages/extra/x86_64/serpl/) using [`pacman`](https://wiki.archlinux.org/title/Pacman):

```bash
pacman -S serpl
```

#### Nix/NixOS

`serpl` is included in [nixpkgs](https://github.com/nixos/nixpkgs) since 24.11, and can be installed via Nix in different ways:

**On standalone Nix setups**:

```bash
nix profile install nixpkgs#serpl
```

**On NixOS** (via `configuration.nix` or similar):

```nix
{pkgs, ...}: {
  environment.systemPackages = [pkgs.serpl];
}
```

**On Home-Manager**:

```nix
{pkgs, ...}: {
  home.packages = [pkgs.serpl];
}
```

## Usage

### Basic Commands

- Start the application in the current directory:
  ```bash
  serpl
  ```
- Start the application and provide the project root path:
  ```bash
  serpl --project-root /path/to/project
  ```

### Key Bindings

Default key bindings can be customized through the `config.json` file.

#### Default Key Bindings

| Key Combination              | Action                                    |
| ---------------------------- | ----------------------------------------- |
| `Ctrl + c`                   | Quit                                      |
| `Ctrl + b`                   | Help                                      |
| `Tab`                        | Switch between tabs                       |
| `Backtab`                    | Switch to previous tabs                   |
| `Ctrl + o`                   | Process replace for all files             |
| `r`                          | Process replace for selected file or line |
| `Ctrl + n`                   | Toggle search and replace modes           |
| `Enter`                      | Execute search (for large folders)        |
| `g` / `Left` / `h`           | Go to top of the list                     |
| `G` / `Right` / `l`          | Go to bottom of the list                  |
| `j` / `Down`                 | Move to the next item                     |
| `k` / `Up`                   | Move to the previous item                 |
| `d`                          | Delete selected file or line              |
| `Esc`                        | Exit the current pane or dialog           |
| `Enter` (in dialogs) / `y`   | Confirm action                            |
| `Esc` (in dialogs) / `n`     | Cancel action                             |
| `h`, `l`, `Tab` (in dialogs) | Navigate dialog options                   |

### Configuration

`serpl` uses a configuration file to manage key bindings and other settings. By default, the path to the configuration file can be found by running `serpl --version`. You can use various file formats for the configuration, such as JSON, JSON5, YAML, TOML, or INI.

#### Example Configurations

<details>
<summary>JSON</summary>
 
```json
{
  "keybindings": {
    "<Ctrl-d>": "Quit",
    "<Ctrl-c>": "Quit",
    "<Tab>": "LoopOverTabs",
    "<Backtab>": "BackLoopOverTabs",
    "<Ctrl-o>": "ProcessReplace"
  }
}
```
</details>
<details>
<summary>JSON5</summary>
 
```json5
{
  keybindings: {
    "<Ctrl-d>": "Quit",
    "<Ctrl-c>": "Quit",
    "<Tab>": "LoopOverTabs",
    "<Backtab>": "BackLoopOverTabs",
    "<Ctrl-o>": "ProcessReplace",
  },
}
```
</details>
<details>
<summary>YAML</summary>
 
```yaml
keybindings:
  "<Ctrl-d>": "Quit"
  "<Ctrl-c>": "Quit"
  "<Tab>": "LoopOverTabs"
  "<Backtab>": "BackLoopOverTabs"
  "<Ctrl-o>": "ProcessReplace"
```
</details>
<details>
<summary>TOML</summary>
 
```toml
[keybindings]
"<Ctrl-d>" = "Quit"
"<Ctrl-c>" = "Quit"
"<Tab>" = "LoopOverTabs"
"<Backtab>" = "BackLoopOverTabs"
"<Ctrl-o>" = "ProcessReplace"
```
</details>
<details>
<summary>INI</summary>
 
```ini
[keybindings]
<Ctrl-d> = Quit
<Ctrl-c> = Quit
<Tab> = LoopOverTabs
<Backtab> = BackLoopOverTabs
<Ctrl-o> = ProcessReplace
```
</details>

You can customize the key bindings by modifying the configuration file in the format of your choice.

## Panes

### Search Input

- Input field for entering search keywords.
- Toggle search modes (Simple, Match Case, Match Whole Word, Match Case Whole Word, Regex, AST Grep).
  - Simple: Search all occurrences of the keyword.
  - Match Case: Search occurrences with the same case as the keyword.
  - Match Whole Word: Search occurrences that match the keyword exactly.
  - Match Case Whole Word: Search occurrences that match the keyword exactly with the same case.
  - Regex: Search occurrences using a regular expression.
  - AST Grep: Search occurrences using AST Grep.
 
> [!TIP] 
> If current directory is considerebly large, you have to click `Enter` to start the search.

### Replace Input

- Input field for entering replacement text.
- Toggle replace modes (Simple, Preserve Case, AST Grep).
  - Simple: Replace all occurrences of the keyword.
  - Preserve Case: Replace occurrences while preserving the case of the keyword.
  - AST Grep: Replace occurrences using AST Grep.

### Search Results Pane

- List of files with search results.
- Navigation to select and view files.
- Option to delete files from the search results.

### Preview Pane

- Display of the selected file with highlighted search results, and context.
- Navigation to view different matches within the file.
- Option to delete individual lines containing matches.

## Quick Hints
- Use the `Ctrl + b` key combination to display the help dialog.
- Use the `Ctrl + o` key combination to process the replace for all files.
- Use the `r` key to process the replace for the selected file or line.
- Use the `Ctrl + n` key combination to toggle between search and replace modes.
- Use the `g`, `G`, `j`, and `k` keys to navigate through the search results.
- Use the `d` key to delete the selected file or line.
 
## Neovim Integration using toggleterm

Check out the [toggleterm.nvim](https://github.com/akinsho/toggleterm.nvim) plugin for Neovim, which provides a terminal that can be toggled with a key binding.
Or you can use the following configuration, if you are using [AstroNvim](https://astronvim.com/):

```lua
return {
  "akinsho/toggleterm.nvim",
  cmd = { "ToggleTerm", "TermExec" },
  dependencies = {
    {
      "AstroNvim/astrocore",
      opts = function(_, opts)
        local maps = opts.mappings
        local astro = require "astrocore"
        maps.n["<Leader>t"] = vim.tbl_get(opts, "_map_sections", "t")

        local serpl = {
          callback = function()
            astro.toggle_term_cmd "serpl"
          end,
          desc = "ToggleTerm serpl",
        }
        maps.n["<Leader>sr"] = { serpl.callback, desc = serpl.desc }

        maps.n["<Leader>tf"] = { "<Cmd>ToggleTerm direction=float<CR>", desc = "ToggleTerm float" }
        maps.n["<Leader>th"] = { "<Cmd>ToggleTerm size=10 direction=horizontal<CR>", desc = "ToggleTerm horizontal split" }
        maps.n["<Leader>tv"] = { "<Cmd>ToggleTerm size=80 direction=vertical<CR>", desc = "ToggleTerm vertical split" }
        maps.n["<F7>"] = { '<Cmd>execute v:count . "ToggleTerm"<CR>', desc = "Toggle terminal" }
        maps.t["<F7>"] = { "<Cmd>ToggleTerm<CR>", desc = "Toggle terminal" }
        maps.i["<F7>"] = { "<Esc><Cmd>ToggleTerm<CR>", desc = "Toggle terminal" }
        maps.n["<C-'>"] = { '<Cmd>execute v:count . "ToggleTerm"<CR>', desc = "Toggle terminal" }
        maps.t["<C-'>"] = { "<Cmd>ToggleTerm<CR>", desc = "Toggle terminal" }
        maps.i["<C-'>"] = { "<Esc><Cmd>ToggleTerm<CR>", desc = "Toggle terminal" }
      end,
    },
  },
  opts = {
    highlights = {
      Normal = { link = "Normal" },
      NormalNC = { link = "NormalNC" },
      NormalFloat = { link = "NormalFloat" },
      FloatBorder = { link = "FloatBorder" },
      StatusLine = { link = "StatusLine" },
      StatusLineNC = { link = "StatusLineNC" },
      WinBar = { link = "WinBar" },
      WinBarNC = { link = "WinBarNC" },
    },
    size = 10,
    ---@param t Terminal
    on_create = function(t)
      vim.opt_local.foldcolumn = "0"
      vim.opt_local.signcolumn = "no"
      if t.hidden then
        local toggle = function() t:toggle() end
        vim.keymap.set({ "n", "t", "i" }, "<C-'>", toggle, { desc = "Toggle terminal", buffer = t.bufnr })
        vim.keymap.set({ "n", "t", "i" }, "<F7>", toggle, { desc = "Toggle terminal", buffer = t.bufnr })
      end
    end,
    shading_factor = 2,
    direction = "float",
    float_opts = { border = "rounded" },
  },
}
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing
(WIP)

## Acknowledgements
- This project was inspired by the [VS Code](https://code.visualstudio.com/) search and replace functionality.
- This project is built using the awesome [ratatui.rs](https://ratatui.rs) library, and build on top of their [Component Template](https://ratatui.rs/templates/component).
- Thanks to the [ripgrep](https://github.com/BurntSushi/ripgrep) project for providing the search functionality.
- Thanks to the [ast-grep](https://ast-grep.github.io) project for providing the AST Grep functionality.

## Similar Projects
- [repgrep](https://github.com/acheronfail/repgrep): An interactive replacer for ripgrep that makes it easy to find and replace across files on the command line.

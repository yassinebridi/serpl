# Serpl

`serpl` is a terminal user interface (TUI) application that allows users to search and replace keywords in an entire folder, similar to the functionality available in VS Code.

https://github.com/yassinebridi/serpl/assets/18403595/e9ae508f-3f12-4633-9c11-2560052a3967

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
4. [Screens](#screens)
   - [Search Screen](#search-screen)
   - [Replace Screen](#replace-screen)
   - [Search Results Screen](#search-results-screen)
   - [Preview Screen](#preview-screen)
5. [Neovim Integration using toggleterm](#neovim-integration-using-toggleterm)
6. [License](#license)
7. [Contributing](#contributing)

## Features

- Search for keywords across an entire project folder.
- Replace keywords with options for preserving case.
- Interactive preview of search results.
- Keyboard navigation for efficient workflow.
- Configurable key bindings and search modes.

## Installation and Update

### Prerequisites

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed on your system.
- [ripgrep](https://github.com/BurntSushi/ripgrep?tab=readme-ov-file#installation) installed on your system.

### Steps

1. Install the application using Cargo:
  ```bash
  cargo install serpl
  ```
2. Update the application using Cargo:
  ```bash
  cargo install serpl
  ```
3. Run the application:
  ```bash
  serpl
  ```
   
### Binaries
Check the [releases](releases) page for the latest binaries.

### OS Specific Installation
(WIP)

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

| Key Combination              | Action                            |
| ---------------------------- | --------------------------------- |
| `Ctrl + c`                   | Quit                              |
| `Tab`                        | Switch between input fields       |
| `Backtab`                    | Switch to previous input field    |
| `Ctrl + o`                   | Process replace                   |
| `Ctrl + n`                   | Toggle search mode                |
| `Ctrl + p`                   | Toggle replace mode               |
| `Enter`                      | Execute search                    |
| `g`                          | Go to top of the list             |
| `G`                          | Go to bottom of the list          |
| `j` / `Down`                 | Move to the next item             |
| `k` / `Up`                   | Move to the previous item         |
| `d`                          | Delete selected file or line      |
| `Esc`                        | Exit the current screen or dialog |
| `Enter` (in dialogs)         | Confirm action                    |
| `h`, `l`, `Tab` (in dialogs) | Navigate dialog options           |

### Configuration

`serpl` uses a `config.json` file to manage key bindings and other settings. By default, this file is located in the project's config directory.

Example `config.json`:

```json
{
  "keybindings": {
    "Normal": {
      "<Ctrl-d>": "Quit",
      "<Ctrl-c>": "Quit",
      "<Ctrl-z>": "Suspend",
      "<Ctrl-r>": "Refresh",
      "<Tab>": "Tab",
      "<Backtab>": "BackTab",
      "<Ctrl-o>": "ProcessReplace"
    }
  }
}
```

You can customize the key bindings by modifying the `config.json` file.

## Screens

### Search Screen

- Input field for entering search keywords.
- Toggle search modes (Simple, Match Case, Whole Word, Regex).

### Replace Screen

- Input field for entering replacement text.
- Toggle replace modes (Simple, Preserve Case).

### Search Results Screen

- List of files with search results.
- Navigation to select and view files.
- Option to delete files from the search results.

### Preview Screen

- Display of the selected file with highlighted search results.
- Navigation to view different matches within the file.
- Option to delete individual lines containing matches.

 
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

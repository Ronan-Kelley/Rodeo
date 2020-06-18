# Rodeo
dotfile manager written from scratch in rust, configured with a single, simple .toml file.

# example configuration file

```TOML
dotfiles_directory = "/path/to/your/local/dotfiles/repo"
[[program]]
name = "nvim"
root = "~/.config/nvim"
paths = ["init.vim", "init-coc.vim"]

[[program]]
name = "bash"
root = "~/"
paths = [".bashrc", ".bash_profile"]
```

disclaimer: programs that scatter their configuration files throughout your system may be somewhat clunky to use with this software.

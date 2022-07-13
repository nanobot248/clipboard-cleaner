# Clipboard Cleaner

Simple Rust / GTK tool to clean up clipboard text.

## Features
  
  * Shows current clipboard targets.
  * Supports several text encodings (UTF-8, UTF-16 with BOM, ISO-8859-1, ...)
  * Includes "Wipe clipboard" function that overwrites the current clipboard content with
    an empty string. Does not really clear the clipboard as this often does not work due to
    clipboard managers (e.g. `klipper` overwrites the clipboard with the last content if it is cleared).
  * Configurable via YAML/JSON/TOML config file
    * Supports defining transformation profiles in order to define which characters
      should be mapped to which output strings.
    * Characters can be mapped to string literals, HTML/XML entities,
      `\x##` hex bytes, `\u####` and `\U########` (`u`/`U` depending on codepoint value),
      `U+#` and `\u{#}`. See `src/assets/default-config.yaml` for examples.

## Configuration

The configuration file is called `clipboard-builder.ext` (ext being one of `yaml`, `json`
or `toml`) and can be set at multiple locations:

 * on Linux, Mac OS, Solaris, Free BSD and OpenBSD:
   * `/etc/clipboard_cleaner`
   * `/etc/clipboard-cleaner`
 * On all operating systems:
   * Subdirectories `conf` and `etc` of the processes current working directory
   * In the operating-system specific application config directory. Clipboard cleaner uses
     the `directories` crate with qualifier `net.laerrus`, company `Laerrus Ultd.` and 
     application name `clipboard-cleaner`. In a typical Linux system with XDG support, the config file
     would be in the directory `~/.config/clipboard-cleaner`.

## Usage

Just start the executable. There are no commandline-arguments. You can exit the application easily
via pressing `ESC`. No other keyboard shortcuts are supported yet.

## Build

The project was developed with Rust version 1.60.0, but will probably work with other versions too.
To build the project, either use `cargo build` or `cargo build --release`. There are custom release
settings configured in `Cargo.toml` to reduce the executable size (currently to less than 1MB on
x86_64) since this project is supposed to be repeatedly started by a keyboard shortcut.
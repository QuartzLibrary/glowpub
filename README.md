# A Glowfic-To-Epub converter

Using this software requires [Rust](https://www.rust-lang.org/tools/install).

You can either clone the repository and follow the instructions, or run `cargo install glowpub --locked` to install `glowpub`. If you install it, replace `cargo run --` with `glowpub` in the commands below.

---

To process a specific post clone this repo and, from its directory, run:
```sh
cargo run -- post <post-id>
```

This will download the entire thread and cache it locally, along with all images.
It'll then generate a single html file in `/books/html/<post-id>.html`, and an epub file in `/books/epub/<post-id>.epub`.

---

To process an entire board/continuity:
```sh
cargo run -- board <board-id>
```

For example, if you want to download all of [planecrash](https://glowfic.com/boards/215):
```sh
cargo run -- board 215
```

---

### Additional options

> Combine these by chaining them after the command.
> 
> ```sh
> cargo run -- board 215 --use-cache --jpeg --text-to-speech --flatten-details=mixed --single-file
> ```

- `--use-cache`: re-use already cached items.
- `--jpeg`: convert images in epubs to jpeg, can lead to significantly smaller file sizes.
- `--resize-icons`: downscale the icons in epubs to the specified width (e.g. `--resize-icons=250`) in pixels, or 100 pixels if unspecified.
- `--text-to-speech`: change the output in a way that may be more comfortable for text-to-speech.
- `--flatten-details`: flatten `details` tags (see example below).
  Valid values are `--flatten-details=none` (default), `--flatten-details=all`, `--flatten-details=mixed`. `mixed` flattens details in epubs only.
- `--output-dir`: output files in this directory (e.g. `--output-dir=~/glowfic`).
  Note that this can flood the directory if used with `board` but without `--single-file`.
- `--single-file`: if downloading a board/continuity, output the entire board in a single epub file.

---

With `flatten-details` enabled this:
> <details>
> <summary>This is a summary</summary>
> This is the hidden content
> </details>

Becomes more or less:

> ▼ This is a summary
> 
> This is the hidden content

Note that you can't close the latter, so the inherent spoiler protection is compromised, this is mostly useful for ereaders that have trouble with tags.
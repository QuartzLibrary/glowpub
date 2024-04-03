# A Glowfic-To-Epub converter

Using this software requires [Rust](https://www.rust-lang.org/tools/install).

---

To process a specific post clone this repo and, from its directory, run:
```
cargo run --example process <post-id>
```

This will download the entire thread and cache it locally, along with all images.
It'll then generate a single html file in `/books/html/<post-id>.html`, and an epub file in `/books/epub/<post-id>.epub`.

---

To process an entire board, run:
```
cargo run --example process_board <board-id>
```

---

To process the entire `planecrash` series:
```
cargo run --example process_board 215
```

---

### Additional options

> Combine these by chaining them after the `--`:
>
> `cargo run --example process_board 215 -- --use-cache --text-to-speech`.

- To re-use already cached items, add `-- --use-cache` to the end of the command.

- To change the output in a way that may be more comfortable for text-to-speech, add `-- --text-to-speech` to the end of the command.

- To flatten `details` tags (see example below) use `-- --flatten-details`.


With `--flatten-details` this:
> <details>
> <summary>This is a summary</summary>
> This is the hidden content
> </details>

Becomes more or less:

> ▼ This is a summary
> 
> This is the hidden content

Note that you can't close the latter, so the inherent spoiler protection is compromised.
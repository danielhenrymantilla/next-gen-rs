#!/usr/bin/env python3

def main():
    with open("README.md", "w") as out:
        with open("src/lib.md", "r") as f:
            lines = f.readlines()
        snippet = False
        for line in lines:
            stripped = line.strip() + " "
            if snippet and stripped.startswith("# "):
                continue
            if snippet and stripped.startswith("##"):
                before, __, after = line.partition("##")
                line = "#".join((before, after))
            if snippet and stripped.startswith("```rust"):
                line = line.split("```rust", 1)[0] + "```rust"
            if stripped.startswith("```"):
                snippet = not snippet
                if snippet:
                    prev, sep, __ = line.partition("```rust")
                    if sep != "":
                        line = "".join((prev, sep, "\n"))
            out.write(line)


if __name__ == '__main__':
    main()

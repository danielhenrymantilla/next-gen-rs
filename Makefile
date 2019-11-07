CARGO=cargo +nightly

.PHONY: default all doc test publish watch clean

default: test

all: publish

publish: README-crates-io.md test
	cargo publish --manifest-path src/proc_macro/Cargo.toml
	sleep 10
	cargo publish

README-crates-io.md: README.md
	sed -e 's/^```rust.*/```rust/' "$<" | tee "$@"

doc: test
	$(CARGO) doc --features external_doc --open

test:
	$(CARGO) test --features external_doc
	@echo "WARNING: miri test is disabled"  # sh ./run_miri.sh

watch:
	cargo watch -c -s "$(CARGO) check --features allow-warnings --profile test && $(CARGO) check --profile test"

clean:
	rm -f README-crates-io.md
	cargo clean

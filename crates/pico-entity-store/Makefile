.PHONY: test bench bench-log lint publish

test:
	cargo nextest run

lint:
	cargo clippy --all-targets -- -D warnings
	cargo check --all-targets

WORKSPACE_ROOT := $(shell git rev-parse --show-toplevel)

bench:
	@rm -rf $(WORKSPACE_ROOT)/target/criterion
	cargo bench --bench ecs_benchmarks
	python3 scripts/benchmark_table.py
	@rm -rf $(WORKSPACE_ROOT)/target/criterion

bench-log:
	python3 scripts/bench_log.py

publish:
	python3 scripts/publish.py

.PHONY: lint test run-desktop watch-desktop run-desktop-release build-desktop build-web run-web

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings -A clippy::module-inception

test:
	cargo nextest run --workspace

build-desktop:
	python clients/desktop/build.py

run-desktop: build-desktop
	python clients/desktop/run.py

watch-desktop:
	python clients/desktop/watch.py

run-desktop-release:
	python clients/desktop/run_release.py

build-web:
	python clients/web/build.py

run-web: build-web
	python clients/web/serve.py

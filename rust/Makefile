# cargo install wasm-pack
.PHONY: js
JS_OUT_DIR := ../pkg

.PHONY: js clean-pkg

js: clean-pkg
	wasm-pack build --target web --no-default-features --features js,zip
	mkdir -p $(JS_OUT_DIR)
	cp -f pkg/rust_futhorc.js $(JS_OUT_DIR)/
	cp -f pkg/rust_futhorc_bg.wasm $(JS_OUT_DIR)/

clean-pkg:
	rm -rf $(JS_OUT_DIR)

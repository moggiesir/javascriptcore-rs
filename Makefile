GIR = gir/target/bin/gir
GIR_SRC != find gir/src -name '*.rs'
GIR_SRC += gir/Cargo.toml gir/Cargo.lock gir/build.rs
GIR_FILES = gir-files/JavaScriptCore-4.0.gir

# Run `gir` generating the bindings
gir : src/auto/mod.rs
gir-sys : javascriptcore-sys/src/lib.rs
gir-gtksys : javascriptcoregtk-sys/src/lib.rs

doc: $(GIR) $(GIR_FILES)
	$(GIR) -m doc -c Gir.toml

not_bound: $(GIR) $(GIR_FILES)
	$(GIR) -m not_bound -c Gir.toml

regen_check: $(GIR) $(GIR_FILES)
	rm src/auto/*
	rm javascriptcore-sys/tests/*
	#rm javascriptcoregtk-sys/tests/*
	$(GIR) -c Gir.toml
	$(GIR) -c javascriptcore-sys/Gir.toml
	#$(GIR) -c javascriptcoregtk-sys/Gir.toml
	git diff -R --exit-code

src/auto/mod.rs : Gir.toml $(GIR) $(GIR_FILES) gir-sys
	$(GIR) -c Gir.toml

javascriptcore-sys/src/lib.rs : javascriptcore-sys/Gir.toml $(GIR) $(GIR_FILES)
	$(GIR) -c javascriptcore-sys/Gir.toml

javascriptcoregtk-sys/src/lib.rs : javascriptcoregtk-sys/Gir.toml $(GIR) $(GIR_FILES)
	$(GIR) -c javascriptcoregtk-sys/Gir.toml

$(GIR) : $(GIR_SRC)
	rm -f gir/target/bin/gir
	cargo install --path gir --root gir/target
	rm -f gir/target/.crates.toml

$(GIR_SRC) $(GIR_FILES) :
	git submodule update --init

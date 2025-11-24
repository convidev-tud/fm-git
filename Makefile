install:
	cargo install --frozen --path .
	mkdir --parents ~/.local/share/bash-completion/completions
	cp completion.sh ~/.local/share/bash-completion/completions/fm-git

test:
	cargo test

clean:
	rm ~/.cargo/bin/fm-git
	rm ~/.local/share/bash-completion/completions/fm-git
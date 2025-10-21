install:
	cargo install --path .
	mkdir --parents ~/.local/share/bash-completion/completions
	cp completion.sh ~/.local/share/bash-completion/completions/fm-git

clean:
	rm ~/.cargo/bin/fm-git
	rm ~/.local/share/bash-completion/completions/fm-git
FROM mcr.microsoft.com/devcontainers/base:noble

RUN <<-'EOF' bash
	set -eu -o pipefail
	apt-get update
	apt-get install -y --no-install-recommends ca-certificates pkg-config libssl-dev
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sed 's#/proc/self/exe#\/bin\/sh#g' | sh -s -- -y --no-update-default-toolchain 1>/dev/null
	rm -rf /var/lib/apt/lists/*
EOF

RUN <<-'EOF' bash
	set -eu -o pipefail
	apt-get update
	apt-get install -y --no-install-recommends git
	git clone --depth=1 https://github.com/romkatv/powerlevel10k.git $HOME/.oh-my-zsh/custom/themes/powerlevel10k
	git clone --depth=1 https://github.com/zsh-users/zsh-autosuggestions $HOME/.oh-my-zsh/custom/plugins/zsh-autosuggestions
	git clone --depth=1 https://github.com/zsh-users/zsh-completions ${ZSH_CUSTOM:=~/.oh-my-zsh/custom}/plugins/zsh-completions
	cp $HOME/.oh-my-zsh/custom/themes/powerlevel10k/config/p10k-lean.zsh $HOME/.p10k.zsh
	sed -i'' -e 's/^plugins.*)/plugins=(git zsh-autosuggestions zsh-completions)/' -e 's,^ZSH_THEME=.*,ZSH_THEME="powerlevel10k/powerlevel10k",' $HOME/.zshrc
	echo '[[ ! -f "$HOME/.p10k.zsh" ]] || source "$HOME/.p10k.zsh"' >> $HOME/.zshrc
	# https://github.com/ohmyzsh/ohmyzsh/issues/7426
	echo 'export LANG=en_US.UTF-8' >> $HOME/.zshrc
EOF

ENV PATH="/opt/riscv/bin:${PATH}"
ENV LD_LIBRARY_PATH="/opt/riscv/lib:${LD_LIBRARY_PATH}"

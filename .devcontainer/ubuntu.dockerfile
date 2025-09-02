ARG IMAGE
ARG TAG

FROM ${IMAGE}:${TAG}

RUN apt update && export DEBIAN_FRONTEND=noninteractive && \
  apt upgrade -y && \
  apt install -y --no-install-recommends \
    sudo  software-properties-common vim less openssl unzip \
    wget git curl gzip tar ninja-build openssh-client

# Install gcc
RUN apt install -y --no-install-recommends gcc g++ gdb

# Install clang
RUN apt install -y --no-install-recommends \
  clang libclang-dev llvm-dev lld libclang-rt-dev \
  clangd clang-tidy clang-format lldb

# Install optional dependencies
RUN apt install -y --no-install-recommends ccache cppcheck

# Install python
RUN apt install -y --no-install-recommends python3 python3-pip pipx black
RUN pipx install cmakelang

# Install lazyvim and dependencies
RUN apt install -y --no-install-recommends ripgrep fzf luarocks fd-find

# Install cmake and utils
RUN apt install -y --no-install-recommends cmake cmake-curses-gui

# Install utils
RUN apt install -y --no-install-recommends xsel

# Install neovim
RUN curl -LO https://github.com/neovim/neovim/releases/latest/download/nvim-linux-x86_64.tar.gz && \
  sudo rm -rf /opt/nvim && \
  sudo tar -C /opt -xzf nvim-linux-x86_64.tar.gz

# Install lazygit
RUN LAZYGIT_VERSION=$(curl -s "https://api.github.com/repos/jesseduffield/lazygit/releases/latest" | \grep -Po '"tag_name": *"v\K[^"]*') && \
  curl -Lo lazygit.tar.gz "https://github.com/jesseduffield/lazygit/releases/download/v${LAZYGIT_VERSION}/lazygit_${LAZYGIT_VERSION}_Linux_x86_64.tar.gz" && \
  tar xf lazygit.tar.gz lazygit && \
  install lazygit -D -t /usr/local/bin/

# Add ubuntu user to sudoers
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

## Cleanup cached apt data we don't need anymore
RUN apt-get autoremove -y && apt-get clean && \
    rm -rf /var/lib/apt/lists/*

ARG USERNAME=ubuntu
USER ubuntu
ENV HOME=/home/$USERNAME
ENV SHELL=/bin/bash
ENV PATH="$PATH:/opt/nvim-linux-x86_64/bin/:$HOME/.local/bin:$HOME/.cargo/bin"
WORKDIR $HOME/dlt-explorer

# Install additional lazyvim dependencies
RUN pipx install ast-grep-cli

# Autocompletion for bash, git, etc (it is customizable)
RUN bash -c "$(curl -fsSL https://raw.githubusercontent.com/ohmybash/oh-my-bash/master/tools/install.sh)"

# Install lazyvim
RUN git clone https://github.com/LazyVim/starter ~/.config/nvim \
    && rm -rf ~/.config/nvim/.git
RUN nvim --headless "+Lazy! sync" +qa


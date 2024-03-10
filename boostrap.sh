#!/bin/bash

# Update and install necessary packages
sudo apt-get update
sudo apt-get install -y tmux tig cargo

# Clone the GitHub repository into a directory named "tmux"
# Replace "username/repository" with the actual path to the GitHub repository
git clone https://github.com/gpakosz/.tmux.git $HOME/tmux

# Create a symbolic link for the .tmux.conf file from the cloned repo to the home directory
ln -s $HOME/tmux/.tmux.conf $HOME/

# Step 1: Download Neovim
NVIM_VERSION="v0.9.5"
DOWNLOAD_URL="https://github.com/neovim/neovim/releases/download/${NVIM_VERSION}/nvim-linux64.tar.gz"
DOWNLOAD_DIR="$HOME/Downloads"
NVIM_TAR="$DOWNLOAD_DIR/nvim.tar.gz"

mkdir -p "$DOWNLOAD_DIR"
curl -L -o "$NVIM_TAR" "$DOWNLOAD_URL"

# Step 2: Extract Neovim
EXTRACT_DIR="$HOME/nvim"
mkdir -p "$EXTRACT_DIR"
tar -xzf "$NVIM_TAR" -C "$EXTRACT_DIR" --strip-components 1

# Step 3: Add Neovim bin to PATH
NVIM_BIN="$EXTRACT_DIR/bin"
echo "export PATH=\$PATH:$NVIM_BIN" >> "$HOME/.bashrc"

# Reload .bashrc to apply changes immediately
source "$HOME/.bashrc"

# Step 4: Install Neovim plugins
git clone https://github.com/LazyVim/starter ~/.config/nvim
rm -rf ~/.config/nvim/.git

echo "Neovim has been installed and added to PATH. Please restart your terminal or source your .bashrc to apply changes."

# Function to check and install Cargo packages
check_and_install() {
  # $1 = package name, $2 = optional version
  if ! cargo install --list | grep -q "^$1 "; then
    echo "Installing $1..."
    if [ -z "$2" ]; then
      cargo install --locked --force "$1"
    else
      cargo install --locked --force "$1" --version "$2"
    fi
  else
    echo "$1 is already installed."
  fi
}

# Utilizing the function for each package
check_and_install cargo-binstall

# alias
echo 'alias dcu="docker compose up -d"' >> ~/.bashrc
echo 'alias dcd="docker compose down"' >> ~/.bashrc
echo 'alias dlf="docker logs -f"' >> ~/.bashrc
echo 'alias up="cd ../"' >> ~/.bashrc
echo 'alias dreset="dcd && yes | docker system prune -a && dcu --build"' >> ~/.bashrc

# path
export EDITOR=nvim
source ~/.bashrc

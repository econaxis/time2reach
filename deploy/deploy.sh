apt-get install -y git build-essential libgdal-dev curl ca-certificates --no-install-recommends

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y --default-toolchain nightly
echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

#git config --global http.sslverify false
#
#git clone https://github.com/econaxis/time2reach.git


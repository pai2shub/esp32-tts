# .ide/Dockerfile

FROM cnbcool/default-dev-env:latest

RUN wget https://cnb.cool/rzhangsan/cloud-native-build/-/releases/download/0.0.0/debian12_etc_apt_sources.list.txt && \
    cp debian12_etc_apt_sources.list.txt /etc/apt/sources.list && \
    rm -rf debian12_etc_apt_sources.list.txt

RUN apt update && apt-get update && \
    apt-get install -y \
        git wget flex bison gperf tree curl file \
        python3 python3-pip python3-venv \
        cmake ninja-build ccache \
        libffi-dev libssl-dev \
        dfu-util libusb-1.0-0 \
        gcc build-essential pkg-config \
        libcap2-bin nginx nginx-full nginx-extras rclone \
        locales && \
    rm -rf /var/lib/apt/lists/*

ENV RUSTUP_DIST_SERVER "https://rsproxy.cn"
ENV RUSTUP_UPDATE_ROOT "https://rsproxy.cn/rustup"

# 安装 rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    . $HOME/.cargo/env

ENV PATH="/root/.cargo/bin:$PATH"

# rust cargo 换源
RUN wget https://cnb.cool/rzhangsan/cloud-native-build/-/releases/download/0.0.0/source.crates-io && \
    cp source.crates-io /root/.cargo/config && \
    rm -rf source.crates-io

# 安装 espup
RUN cargo install espup --locked

# 安装其他工具
RUN cargo install cargo-espflash espflash ldproxy cargo-generate

RUN export GITHUB_TOKEN="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" && espup install && \
    tree /root/.rustup/toolchains/esp

RUN code-server --install-extension rust-lang.rust-analyzer \
    && code-server --install-extension tamasfe.even-better-toml \
    && code-server --install-extension usernamehw.errorlens \
    && code-server --install-extension chanhx.crabviz

RUN ARCH=$($HOME/.cargo/bin/rustup show | grep "Default host" | sed -e 's/.* //') && \
    curl -L "https://github.com/esp-rs/esp-web-flash-server/releases/latest/download/web-flash-${ARCH}.zip" -o "${HOME}/.cargo/bin/web-flash.zip" && \
    unzip "${HOME}/.cargo/bin/web-flash.zip" -d "${HOME}/.cargo/bin/" && \
    rm "${HOME}/.cargo/bin/web-flash.zip" && \
    chmod u+x "${HOME}/.cargo/bin/web-flash"

RUN wget https://cnb.cool/rzhangsan/rust-camp-s02/esp-env/-/lfs/6e38a1042c66462a6db5c20b1b54ebb2111977aebf09d729e466299e5086b940?name=embuild.tar.xz -O embuild.tar.gz && \
    # tar -czvf embuild.tar.xz .embuild
    tar -xzvf embuild.tar.gz && rm -rf embuild.tar.gz

# 下载、解压、安装 wstunnel 并清理下载文件
RUN curl -fLo wstunnel.tar.gz https://github.com/erebe/wstunnel/releases/download/v10.4.4/wstunnel_10.4.4_linux_amd64.tar.gz && \
    tar -xzf wstunnel.tar.gz wstunnel && chmod +x wstunnel && \
    mv wstunnel /usr/local/bin && \
    rm wstunnel.tar.gz

# 设置 wstunnel 的功能位
RUN setcap cap_net_bind_service=+ep /usr/local/bin/wstunnel

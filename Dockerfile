FROM debian:jessie
MAINTAINER Andrey Ivanov

ENV RUST_VERSION=1.7.0
ENV REPOSITORY_OMIM=https://github.com/65apps/omim.git
ENV REPOSITORY_GENERATOR=https://github.com/65apps/rust-pbf-to-mvm.git
ENV DIR=/srv
ENV FILES_DIR=/mnt/files/

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    wget \
    git \
    nano \
    libssl-dev \
    clang \
    libc++-dev \
    libglu1-mesa-dev \
    libstdc++-4.8-dev \
    qt5-default \
    cmake \
    libboost-all-dev \
    mesa-utils \
    libtbb2 \
    libtbb-dev \
    libluabind-dev \
    libluabind0.9.1 \
    lua5.1 \
    osmpbf-bin \
    libprotobuf-dev \
    libstxxl-dev \
    libxml2-dev \
    libsparsehash-dev \
    libbz2-dev \
    zlib1g-dev \
    libzip-dev \
    libgomp1 \
    liblua5.1-0-dev \
    pkg-config \
    libgdal-dev \
    libexpat1-dev \
    libosmpbf-dev	

WORKDIR $DIR

RUN mkdir $FILES_DIR

RUN wget https://static.rust-lang.org/dist/rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz

RUN tar -xzf rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz && \
    rust-$RUST_VERSION-x86_64-unknown-linux-gnu/install.sh --without=rust-docs && \
    rm -rf \
        rust-$RUST_VERSION-x86_64-unknown-linux-gnu \
        rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz

WORKDIR $DIR

RUN git clone $REPOSITORY_GENERATOR && \    
    cd rust-pbf-to-mvm && \
    cargo build && \    
    wget https://github.com/github/git-lfs/releases/download/v1.2.0/git-lfs-linux-amd64-1.2.0.tar.gz && \
    tar -xzf git-lfs-linux-amd64-1.2.0.tar.gz && \    
    cd git-lfs-1.2.0 && \
    ./install.sh && \
    cd ../ && \
    git lfs install && \
    git lfs pull && \
    rm -rf git-lfs-1.2.0 git-lfs-linux-amd64-1.2.0.tar.gz

WORKDIR $DIR/rust-pbf-to-mvm

RUN git clone --depth=1 --recursive $REPOSITORY_OMIM

RUN cd omim && \
    echo | ./configure.sh 

RUN CONFIG=gtool omim/tools/unix/build_omim.sh -cro

WORKDIR $DIR/rust-pbf-to-mvm

CMD ["/bin/bash"]
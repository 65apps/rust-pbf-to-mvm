FROM debian:jessie
MAINTAINER Andrey Ivanov stayhardordie@gmail.com

ENV RUST_VERSION=1.7.0
ENV REPOSITORY=https://github.com/mapsme/omim.git
ENV REPOSITORY_GENERATOR=https://github.com/stalehard/rust-pbf-to-mvm.git
ENV DIR=/srv
ENV OMIM_DIR=/srv/omim/tools/unix/generate_mwm.sh
ENV FILES_DIR=/mnt/files/

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    wget \
    git \
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
RUN wget https://static.rust-lang.org/dist/rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz
RUN tar -xzf rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz && \
    rust-$RUST_VERSION-x86_64-unknown-linux-gnu/install.sh --without=rust-docs && \
    rm -rf \
        rust-$RUST_VERSION-x86_64-unknown-linux-gnu \
        rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz
RUN git clone --depth=1 --recursive $REPOSITORY
RUN cd omim && \
    echo | ./configure.sh
WORKDIR $DIR
RUN CONFIG=gtool omim/tools/unix/build_omim.sh -cro
RUN git clone $REPOSITORY_GENERATOR && \    
    cd rust-pbf-to-mvm && \
    cargo build
CMD ["/bin/bash"]

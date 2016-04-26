FROM debian:jessie
MAINTAINER Andrey Ivanov

ENV RUST_VERSION=1.7.0
ENV REPOSITORY_OMIM=https://github.com/65apps/omim.git
ENV REPOSITORY_GENERATOR=https://github.com/65apps/rust-pbf-to-mvm.git
ENV REPOSITORY_GRAPH=https://github.com/graphhopper/graphhopper.git
ENV DIR=/srv
ENV FILES_DIR=/mnt/files/
ENV GRAPH_DIR=/srv/graphhopper/

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

RUN \
    echo "===> add webupd8 repository..."  && \
    echo "deb http://ppa.launchpad.net/webupd8team/java/ubuntu trusty main" | tee /etc/apt/sources.list.d/webupd8team-java.list  && \
    echo "deb-src http://ppa.launchpad.net/webupd8team/java/ubuntu trusty main" | tee -a /etc/apt/sources.list.d/webupd8team-java.list  && \
    apt-key adv --keyserver keyserver.ubuntu.com --recv-keys EEA14886  && \
    apt-get update  && \
    \
    \
    echo "===> install Java"  && \
    echo debconf shared/accepted-oracle-license-v1-1 select true | debconf-set-selections  && \
    echo debconf shared/accepted-oracle-license-v1-1 seen true | debconf-set-selections  && \
    DEBIAN_FRONTEND=noninteractive  apt-get install -y --force-yes oracle-java8-installer oracle-java8-set-default maven && \
    \
    \
    echo "===> clean up..."  && \
    rm -rf /var/cache/oracle-jdk8-installer  && \
    apt-get clean  && \
    rm -rf /var/lib/apt/lists/*

RUN git clone $REPOSITORY_GRAPH && \
    cd graphhopper && \
    git checkout 0.5 

WORKDIR $DIR

RUN git clone $REPOSITORY_GENERATOR && \    
    cd rust-pbf-to-mvm && \
    cargo build && \
    mv ./config.properties ../graphhopper/config.properties && \
    wget https://github.com/github/git-lfs/releases/download/v1.2.0/git-lfs-linux-amd64-1.2.0.tar.gz && \
    tar -xzf git-lfs-linux-amd64-1.2.0.tar.gz && \    
    cd git-lfs-1.2.0 && \
    ./install.sh && \
    cd ../ && \
    git lfs install && \
    git lfs pull && \
    rm -rf git-lfs-1.2.0 git-lfs-linux-amd64-1.2.0.tar.gz

RUN git clone --depth=1 --recursive $REPOSITORY_OMIM

RUN cd omim && \
    echo | ./configure.sh && \   
    CONFIG=gtool omim/tools/unix/build_omim.sh -cro

WORKDIR $DIR/rust-pbf-to-mvm

CMD ["/bin/bash"]

FROM ubuntu:14.04
MAINTAINER Andrey Ivanov

ENV RUST_VERSION=1.8.0
ENV REPOSITORY_OMIM=https://github.com/65apps/omim.git
ENV REPOSITORY_GENERATOR=https://github.com/65apps/rust-pbf-to-mvm.git
ENV GENERATOR=rust-pbf-to-mvm/
ENV DIR=/srv
ENV FILES_DIR=/mnt/files/

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    libgl1-mesa-dev \
    wget \
    git subversion \
    nano \
    clang-3.6 \
    libbz2-dev \
    libssl-dev \
    libc++-dev \
    libboost-iostreams-dev \
    libglu1-mesa-dev \
    libtbb2 \
    libluabind0.9.1 \
    liblua50 \
    libstxxl1 \
    libtbb-dev \
    libluabind-dev \
    libstxxl-dev \
    libosmpbf-dev \
    libprotobuf-dev \
    libboost-all-dev \
    qt5-default

RUN ln -s /usr/lib/llvm-3.6/bin/clang /usr/bin/clang && \
    ln -s /usr/lib/llvm-3.6/bin/clang++ /usr/bin/clang++

RUN mkdir $FILES_DIR

WORKDIR $DIR

RUN git clone --depth=1 --recursive $REPOSITORY_OMIM && \
    cd omim && \
    echo | ./configure.sh

RUN CONFIG=gtool omim/tools/unix/build_omim.sh -cro

RUN wget https://static.rust-lang.org/dist/rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz

RUN tar -xzf rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz && \
    rust-$RUST_VERSION-x86_64-unknown-linux-gnu/install.sh --without=rust-docs && \
    rm -rf \
        rust-$RUST_VERSION-x86_64-unknown-linux-gnu \
        rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz

RUN wget http://www.cmake.org/files/v3.5/cmake-3.5.2.tar.gz && \
    tar xf cmake-3.5.2.tar.gz && \
    cd cmake-3.5.2 && \
    ./configure && \
    make && make install && \
    ln -sf /srv/cmake-3.5.2/bin/cmake /usr/bin/cmake

RUN svn co http://llvm.org/svn/llvm-project/llvm/trunk llvm && \
    cd llvm/projects && \
    svn co http://llvm.org/svn/llvm-project/libcxx/trunk libcxx && \
    svn co http://llvm.org/svn/llvm-project/libcxxabi/trunk libcxxabi && \
    cd .. && \
    mkdir build && cd build && \
    cmake $DIR/llvm && \
    make cxx && \
    make install-libcxx install-libcxxabi 

RUN rm -r /usr/include/c++/v1/ && \
    mv llvm/build/include/__cxxabi_config.h /usr/include/ && \
    mv llvm/build/include/cxxabi.h /usr/include/ && \
    mv llvm/build/include/c++/v1 /usr/include/c++/

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

RUN git clone $REPOSITORY_GENERATOR && \
    cd $GENERATOR && \
    cargo build && \
    wget https://github.com/github/git-lfs/releases/download/v1.2.0/git-lfs-linux-amd64-1.2.0.tar.gz && \
    tar -xzf git-lfs-linux-amd64-1.2.0.tar.gz && \
    cd git-lfs-1.2.0 && \
    ./install.sh && \
    cd ../ && \
    git lfs install && \
    git lfs pull && \
    rm -rf git-lfs-1.2.0 git-lfs-linux-amd64-1.2.0.tar.gz

WORKDIR $DIR/$GENERATOR

CMD ["/bin/bash"]
# A Linux build of metorex, in the image CI runs on, so a failure that shows
# only on Linux can be reproduced without waiting for CI.
#
# The repository is mounted at /work when the container runs rather than
# copied in, so a build here reads the working tree as it stands.
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# `binutils` carries the tools RbConfig names, such as the one that strips a
# binary. `libcrypt-dev` carries the library `String#crypt` links against,
# which Linux keeps separate from the C library. `libssl-dev` is what the
# coverage tool builds against. `netbase` carries /etc/services, which a port
# named by its service reads. `tzdata` and `tzdata-legacy` carry the zones a
# time in a named place is read against, down to the older names a spec still
# writes. `libyaml-dev` is what Ruby loads YAML through.
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
       binutils \
       build-essential \
       ca-certificates \
       curl \
       libcrypt-dev \
       libssl-dev \
       libyaml-dev \
       netbase \
       pkg-config \
       procps \
       time \
       tzdata \
       tzdata-legacy \
  && rm -rf /var/lib/apt/lists/*

# The mspec driver runs under Ruby, and the vendored suite is written for the
# release metorex reports itself as. Ubuntu carries no package for it, so the
# build the CI runner installs is unpacked here, which is the same Ruby by the
# same route rather than one built from source.
ARG RUBY_VERSION=4.0.1
ARG TARGETARCH
RUN case "$TARGETARCH" in \
      amd64) built_for=x64 ;; \
      arm64) built_for=arm64 ;; \
      *) echo "no prebuilt Ruby for $TARGETARCH" >&2; exit 1 ;; \
    esac \
  && mkdir -p "/opt/hostedtoolcache/Ruby/$RUBY_VERSION" \
  && curl -fsSL \
       "https://github.com/ruby/ruby-builder/releases/download/ruby-$RUBY_VERSION/ruby-$RUBY_VERSION-ubuntu-24.04-$built_for.tar.gz" \
    | tar xz -C "/opt/hostedtoolcache/Ruby/$RUBY_VERSION" \
  && ln -s "/opt/hostedtoolcache/Ruby/$RUBY_VERSION/$built_for" /usr/local/ruby

# The build is made for the place the runner keeps it, which is where it is
# unpacked, so what it holds is found without being told where to look.
ENV PATH=/usr/local/ruby/bin:$PATH

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
  | sh -s -- -y --default-toolchain stable --profile minimal \
       --component rustfmt --component clippy

# The container runs as an account of its own rather than as root, since a
# test that asks what a file refuses passes for root whatever the file says.
# The account is given the ids the host builds under, so what it writes to the
# mounted tree belongs to whoever started it, and it is a real account in the
# password database, which `Etc.getpwuid` and `File.expand_path "~"` read.
ARG UID=1000
ARG GID=1000
# The base image already has an `ubuntu` account with id 1000, the id a Linux
# host's first user and the Harness CI checkout both build under, so whatever
# account holds the id is removed first.
RUN if existing=$(getent passwd "$UID"); then userdel --remove "${existing%%:*}"; fi \
  && if ! getent group "$GID" >/dev/null; then groupadd --gid "$GID" builder; fi \
  && useradd --uid "$UID" --gid "$GID" --create-home --shell /bin/bash builder

# Coverage compiles the tests its own way, so the tool CI measures with is
# part of the image rather than something to install when a run needs it.
RUN cargo install cargo-tarpaulin --locked

# The toolchain is left writable so that account can build with it, and the
# registry belongs to it in the image, so the volume mounted over that path
# when the container runs takes the same ownership.
RUN chmod -R a+rwX "$CARGO_HOME" "$RUSTUP_HOME" \
  && mkdir -p "$CARGO_HOME/registry" \
  && chown -R "$UID:$GID" "$CARGO_HOME/registry"

# The build directory and the directory the spec suite compiles C extensions
# into belong to that account in the image, so the volume mounted over each
# when the container runs takes the same ownership and can be written to.
RUN mkdir -p /work/target /work/ruby/spec/ext \
  && chown "$UID:$GID" /work /work/target /work/ruby /work/ruby/spec /work/ruby/spec/ext

USER builder
WORKDIR /work
CMD ["cargo", "test"]

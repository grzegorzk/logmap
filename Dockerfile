ARG GROUP_ID=1001
ARG USER_ID=1001

FROM docker.io/techgk/arch:latest AS logmap

RUN pacman -Sy --disable-download-timeout --noconfirm \
        rust \
    && rm -rf /var/cache/pacman/pkg/*

COPY src /opt/logmap/src
COPY Cargo.toml /opt/logmap/Cargo.toml

RUN cd /opt/logmap \
    && cargo build --release

ENTRYPOINT ["/opt/logmap/target/release/logmap"]
CMD ["--help"]

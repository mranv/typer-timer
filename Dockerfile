FROM rust:1.73.0 AS build

RUN apt-get update && \
    apt-get install -y libxi-dev libxtst-dev && \
    rm -rf /var/lib/apt/lists/*
RUN rustup component add rustfmt

WORKDIR /workspaces/typer-timer

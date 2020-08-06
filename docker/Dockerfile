FROM phusion/baseimage:0.11 as builder
LABEL maintainer "dappforce@pm.me"

WORKDIR /subsocial

COPY . /subsocial

RUN apt-get update && \
	apt-get upgrade -y && \
	apt-get install -y cmake pkg-config libssl-dev git clang
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
        export PATH=$PATH:$HOME/.cargo/bin && \
        scripts/init.sh && \
        cargo build --release

FROM phusion/baseimage:0.11
LABEL maintainer "dappforce@pm.me"

COPY --from=builder /subsocial/target/release/subsocial-node /usr/local/bin

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	rm -rf /usr/lib/python* && \
	useradd -m -u 1000 -U -s /bin/sh -d /subsocial subsocial && \
	mkdir -p /subsocial/.local/share/subsocial-node && \
	chown -R subsocial:subsocial /subsocial/.local && \
	ln -s /subsocial/.local/share/subsocial-node /data && \
	rm -rf /usr/bin /usr/sbin

COPY ./node/res/customSpecRaw.json /subsocial/customSpecRaw.json

USER subsocial
EXPOSE 30333 9933 9944
VOLUME ["/data"]

CMD ["/usr/local/bin/subsocial-node --chain=/subsocial/customSpecRaw.json"]

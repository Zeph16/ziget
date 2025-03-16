FROM rust:latest

WORKDIR /ziget

COPY . /ziget

RUN apt-get update && apt-get install -y lsb-release wget software-properties-common gnupg
RUN wget https://apt.llvm.org/llvm.sh
RUN chmod +x llvm.sh
RUN ./llvm.sh 18 all
RUN cargo build --release
RUN cp target/release/ziget /usr/local/bin/ziget
RUN chmod +x /usr/local/bin/ziget

ENV ZIGET_CLANG_PATH="clang-18"

CMD ziget playground/main.zg --lexer-output --parser-output --symbol-output && \
    echo '----------------- Executing main.out -----------------' && \
    echo && \
    ./playground/main.out || true && \
    echo && \
    echo '----------------- Finished executing main.out -----------------'

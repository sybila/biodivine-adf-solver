# For some reason, PyEDA does not like newer Python and/or Debian versions.
FROM python:3.11-bookworm

WORKDIR /sw

# Core dependencies needed for benchmarking:
RUN apt-get update && apt install -y --no-install-recommends coreutils time && rm -rf /var/lib/apt/lists/*

# Tool-specific dependencies:
RUN apt-get update && apt install -y --no-install-recommends build-essential gringo unzip && rm -rf /var/lib/apt/lists/*

# Downloaded from https://github.com/daemontus/tsconj
COPY ./tsconj-main.zip /sw/tsconj-main.zip
RUN unzip ./tsconj-main.zip

RUN python3 -m venv ./venv
RUN /sw/venv/bin/pip install ./tsconj-main

COPY ./run.py /sw/run.py

# A designated file that benchmark scripts will consider as entry point.
COPY ./run.sh /run.sh
RUN chmod +x /run.sh

WORKDIR /root

CMD ["/bin/bash"]
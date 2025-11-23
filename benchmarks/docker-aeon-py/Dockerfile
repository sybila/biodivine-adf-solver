FROM debian:trixie

WORKDIR /sw

# Core dependencies needed for benchmarking:
RUN apt-get update && apt install -y --no-install-recommends coreutils time && rm -rf /var/lib/apt/lists/*

# Tool-specific dependencies:
RUN apt-get update && apt install -y --no-install-recommends python3 python3-venv && rm -rf /var/lib/apt/lists/*

RUN python3 -m venv ./venv
RUN /sw/venv/bin/pip install biodivine_aeon==1.2.5

# A designated file that benchmark scripts will consider as entry point.
COPY ./run.sh /run.sh
RUN chmod +x /run.sh

# Helper scripts that are useful to have in the docker container (but it is possible to run your own script too)
COPY ./adf_complete.py /sw/adf_complete.py
COPY ./adf_preferred.py /sw/adf_preferred.py
COPY ./adf_two_valued.py /sw/adf_two_valued.py

WORKDIR /root

CMD ["/bin/bash"]
FROM debian:trixie

WORKDIR /sw

# Core dependencies needed for benchmarking:
RUN apt-get update && apt install -y --no-install-recommends coreutils time && rm -rf /var/lib/apt/lists/*

# Tool-specific dependencies:
RUN apt-get update && apt install -y --no-install-recommends gringo default-jre && rm -rf /var/lib/apt/lists/*

# Obtained from https://github.com/colomoto/bioLQM/releases/tag/v0.6.1
COPY ./bioLQM-0.6.1.jar /sw/bioLQM-0.6.1.jar

# A designated file that benchmark scripts will consider as entry point.
COPY ./run.sh /run.sh
RUN chmod +x /run.sh

WORKDIR /root

CMD ["/bin/bash"]
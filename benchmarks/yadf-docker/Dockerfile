FROM debian:trixie

WORKDIR /sw

RUN apt-get update && apt install -y --no-install-recommends wget gringo default-jre && rm -rf /var/lib/apt/lists/*

# RUN wget https://www.dbai.tuwien.ac.at/proj/adf/yadf/files/yadf_0.1.1.jar
# RUN wget https://www.dbai.tuwien.ac.at/research/project/lpopt/lpopt-2.2-x86_64.tar.gz

# There seem to be some HTTPS issues with the URLs, so we'll just use a local copy instead...
COPY ./yadf_0.1.1.jar /sw/yadf_0.1.1.jar
COPY ./lpopt-2.2-x86_64.tar.gz /sw/lpopt-2.2-x86_64.tar.gz
COPY ./run.sh /sw/run.sh

RUN chmod +x /sw/run.sh
RUN tar -xzf lpopt-2.2-x86_64.tar.gz

WORKDIR /root

CMD ["/bin/bash"]
# this_file: oss-fuzz/Dockerfile

FROM gcr.io/oss-fuzz-base/base-builder-rust

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the project source
COPY . $SRC/vexy_json

# Set the working directory
WORKDIR $SRC/vexy_json

# Copy the build script
COPY oss-fuzz/build.sh $SRC/build.sh

# Make the build script executable
RUN chmod +x $SRC/build.sh
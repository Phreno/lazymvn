FROM gitpod/workspace-full:latest

# Install additional tools
RUN sudo apt-get update && sudo apt-get install -y \
    git-flow \
    tree \
    htop \
    build-essential \
    pkg-config \
    libssl-dev

# Install SDKMAN for Java/Maven management  
RUN curl -s "https://get.sdkman.io" | bash

# Set environment
ENV JAVA_HOME=/home/gitpod/.sdkman/candidates/java/current
ENV PATH="$JAVA_HOME/bin:$PATH"
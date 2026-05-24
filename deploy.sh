#!/bin/bash

# Configuration
if [ -z "$REMOTE_SERVER_IP" ]; then
    echo -e "${RED}Error: Environment variable REMOTE_SERVER_IP is not set.${NC}"
    echo -e "Please set it before running this script (e.g. export REMOTE_SERVER_IP=\"135.181.27.105\")."
    exit 1
fi
SERVER="$REMOTE_SERVER_IP"
REMOTE_USER="root"
REMOTE_DIR="/root/mattmagie"

# Colors for terminal output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Ensure we are in the script's directory
cd "$(dirname "$0")"

echo -e "${YELLOW}Starting deployment of Matt-Magie and engines to ${SERVER}...${NC}"

# 1. Create remote directory structure
echo -e "${YELLOW}Creating remote directory structure on server...${NC}"
ssh ${REMOTE_USER}@${SERVER} "mkdir -p ${REMOTE_DIR}/engines/"
if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Failed to connect or create directories on ${SERVER}. Check your SSH connection.${NC}"
    exit 1
fi

# 2. Upload source code, config files, and shell scripts
echo -e "${YELLOW}Uploading source code, config files, and shell scripts...${NC}"
# Upload Cargo files and scripts
scp Cargo.toml mm.sh summary.sh summary.py ${REMOTE_USER}@${SERVER}:${REMOTE_DIR}/

# Upload src folder recursively
scp -r src ${REMOTE_USER}@${SERVER}:${REMOTE_DIR}/

# 3. Compile natively on the remote ARM server
echo -e "${YELLOW}Compiling Matt-Magie natively on the remote ARM server...${NC}"
ssh ${REMOTE_USER}@${SERVER} "source \$HOME/.cargo/env && cd ${REMOTE_DIR} && rm -f Cargo.lock && cargo build --release"
if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Remote native compilation failed!${NC}"
    exit 1
fi

# 4. Make scripts and binary executable on the server
echo -e "${YELLOW}Setting executable permissions on the server...${NC}"
ssh ${REMOTE_USER}@${SERVER} "chmod +x ${REMOTE_DIR}/mm.sh ${REMOTE_DIR}/summary.sh ${REMOTE_DIR}/target/release/Matt-Magie && (chmod +x ${REMOTE_DIR}/engines/* 2>/dev/null || true)"

echo -e "\n${GREEN}Deployment and remote ARM compilation completed successfully!${NC}"
echo -e "You can now run Matt-Magie on the server by running:"
echo -e "  ${GREEN}ssh ${REMOTE_USER}@${SERVER} \"cd ${REMOTE_DIR} && ./mm.sh\"${NC}"

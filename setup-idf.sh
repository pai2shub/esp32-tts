#!/bin/bash
export IDF_PATH=/.embuild/espressif/esp-idf/v5.4.1
export IDF_TOOLS_PATH=/.embuild/espressif
export IDF_PYTHON_ENV_PATH=/.embuild/espressif/python_env/idf5.4_py3.11_env

/.embuild/espressif/python_env/idf5.4_py3.11_env/bin/python /.embuild/espressif/esp-idf/v5.4.1/tools/idf_tools.py export --prefer-system

source /.embuild/espressif/esp-idf/v5.4.1/export.sh

source /root/export-esp.sh

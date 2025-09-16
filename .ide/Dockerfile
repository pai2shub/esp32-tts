# .ide/Dockerfile

FROM docker.cnb.cool/rzhangsan/rust-camp-s02/esp32-tts/dockerfile-caches:89c8efba1783aab328f085b3c105e26fb4eddaf2

ENV IDF_PATH "/.embuild/espressif/esp-idf/v5.4.1"
ENV IDF_TOOLS_PATH "/.embuild/espressif"
ENV IDF_PYTHON_ENV_PATH "/.embuild/espressif/python_env/idf5.4_py3.11_env"

RUN /.embuild/espressif/python_env/idf5.4_py3.11_env/bin/python /.embuild/espressif/esp-idf/v5.4.1/tools/idf_tools.py install

RUN source /.embuild/espressif/esp-idf/v5.4.1/export.sh

# clan-code-extension
RUN code-server --uninstall-extension cloudstudio.live-server && \
    code-server --uninstall-extension tencent-cloud.coding-copilot && \
    code-server --uninstall-extension cnbcool.cnb-welcome

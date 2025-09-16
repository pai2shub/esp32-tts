@REM --partition-table partitions.csv 指定分区表文件
@REM --flash-size 16mb 指定 flash 大小
espflash flash --monitor --partition-table partitions.csv --flash-size 16mb .\etts

@REM 设置环境变量
set IDF_PATH=D:\ProgramData\Rust\.rustup\toolchains\esp\.embuild\espressif\esp-idf\v5.4.1
set IDF_PYTHON_PATH=D:\ProgramData\Rust\.rustup\toolchains\esp\.embuild\espressif\python_env\idf5.4_py3.9_env

@REM 激活 python 虚拟环境
%IDF_PYTHON_PATH%\Scripts\Activate.ps1

@REM 设置 esptool 路径
set ESPTOOL=%IDF_PATH%\components\esptool_py\esptool\esptool.py

@REM 查看帮助信息
%ESPTOOL% --help

@REM 烧录数据
@REM 偏移 0x710000 可以通过烧录 partitions.csv 文件后启动一次查看偏移
%ESPTOOL% --before default_reset --after hard_reset write_flash --flash_mode dio --flash_freq 40m --flash_size detect 0x710000 ".\model\esp_tts_voice_data_xiaoxin.dat"

@REM monitor
espflash monitor

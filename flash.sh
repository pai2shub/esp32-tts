#!/bin/bash

echo $IDF_PATH

espflash flash --monitor --partition-table partitions.csv --flash-size 16mb etts

python ${IDF_PATH}/components/esptool_py/esptool/esptool.py \
        --before default_reset --after hard_reset write_flash \
        --flash_mode dio --flash_freq 40m --flash_size detect 0x710000 "./model/esp_tts_voice_data_xiaoxin.dat"

espflash monitor

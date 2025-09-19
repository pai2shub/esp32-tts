ESP32-S3 TTS demo
===


### quick start

- fork this repo
- click "云原生开发"
- VSCode open remote project
- see `CNB_VSCODE_PROXY_URI` file, use `rclone-webdav endpoint` webDAV mount to local pc
- build vscode terminal `./build.sh`
- flash local pc terminal `flash.sh` or `flash.bat`

### other

#### components_xxx.lock

components_esp32s3.lock 在 build 过程中会自动生成

#### config tts

sdkconfig.defaults 需要添加

```
CONFIG_SR_NSN_NSNET2=y
CONFIG_SR_VADN_VADNET1_MEDIUM=y
```

Cargo.toml 需要添加

```toml
[[package.metadata.esp-idf-sys.extra_components]]
remote_component = { name = "espressif/esp-sr", version = "^2.0.0" }
bindings_header = "components/esp_sr/bindgen.h"
bindings_module = "esp_sr"
```

同时需要在 `components/esp_sr/bindgen.h` 添加需要使用的模型头文件

```h
#include "esp_tts.h"
#include "esp_tts_voice_template.h"
#include "esp_tts_voice_xiaole.h"
```


#### config lvgl

- 字体转化为 lvgl 代码 https://lvgl.io/tools/fontconverter
- 将生成的 .c 代码导入到项目中，例如 `custom-fonts2/simhei.c`
- 根据生成的 .c 代码编写对应的 .h 文件，例如 `custom-fonts2/simhei.h`
- 在 lvgl.h 中指定 使用的字体
- 在 `.cargo/config.toml` 中指定使用的字体所属文件夹

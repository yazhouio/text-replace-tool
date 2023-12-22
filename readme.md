### ks console web prefix 替换工具

#### 1. 介绍
本工具特为ks console web前端项目开发，提供了一键替换 prefix 的功能，方便在服务器上部署时，自动替换prefix的功能。

#### 2. 使用说明
```bash
    $text-replace --help
# Options:
#  -c, --config-path <CONFIG_PATH>  config file path [default: config.toml]
#  -h, --help                       Print help
#  -V, --version                    Print version


    $text-replace -c config.toml

```

#### 3. 配置文件说明
```toml

old_web_prefix = '/dca768b9-629c-4b38-a652-1fdee6afbe3e' # 现有的 web prefix，kse embed package.json 中设置
# new_web_prefix = '/k/'   # 新的 web prefix
local_config_path = '/Users/yazhou/code/rust/text-replace/dist/local_config.yaml'  # local_config.
config_path = '/Users/yazhou/code/rust/text-replace/dist/config.yaml' # config.yaml
source_path = '/Users/yazhou/code/rust/text-replace/dist' # 源码路径
output_path = '/Users/yazhou/code/rust/text-replace/output' # 输出路径
file_types = ['js', 'css'] # 需要替换的文件类型
exclude_path = [] # 排除的路径, 相对路径，iter 为正则
```

> 注：new_web_prefix 配置优先级高于 local_config_path 中的配置， local_config_path 高于 config_path 中的配置

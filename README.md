# luogu-printing-tools

使用 Rust 语言编写的 [洛谷冬日绘板](https://www.luogu.com.cn/paintboard) 自动化绘图程序。

*注：因为今年的接口尚未开放，本程序目前仍然使用去年的接口。待接口开放后会及时更新程序与文档。*

## 安装

你首先需要在本地安装 Rust 环境。

如果你使用的是 Linux，macOS 或其他类 Unix 系统，可以使用 [Rustup](https://www.rust-lang.org/zh-CN/tools/install) 安装。

如果你使用的是 Windows 系统，请在 [这里](https://forge.rust-lang.org/infra/other-installation-methods.html#other-ways-to-install-rustup) 找到符合您系统环境的安装程序。

在安装完 Rust 环境后，执行如下命令将本仓库克隆到本地：

```bash
git clone https://github.com/woshiluo/luogu-printing-tools.git # HTTPS
git clone git@github.com:woshiluo/luogu-printing-tools.git # SSH
```

## 配置文件

程序运行所需的配置文件放在 `config.toml` 下。

各参数的含义如下：

- `board_addr`：绘板主页的地址；
- `websocket_addr`：WebSocket API 地址；
- `cookie_dir`：Cookies 存放的文件夹；
- `node_file`：要绘制的图案的数据文件（详情见后文）；
- `wait_time`：单个 Cookies 在两次绘图之间所需的冷却时间；
- `thread_num`：绘图时使用的最大线程数；
- `board_width`：绘板的宽度；
- `board_height`：绘板的高度。

## 绘图数据

绘图数据为 JSON 格式文件，格式如下：

```json
[
    [
        0, // x 坐标
        0, // y 坐标
        0  // 颜色编号
    ],
    [
        0, // 下一个点的信息
        1,
        1
    ]
]
```

## Cookies 数据

Cookies 存放在配置中 `cookie_dir` 对应的文件夹下，文件夹下一个文件对应一个 Cookies。

Cookies 请按如下格式填写：

```json
{
    "cookie": "_uid=x;__client_id=xxxxxxxx"
}
```

## 运行

在完成以上配置后，执行 `cargo run` 即可启动程序。

运行时产生的全部日志信息会输出到标准错误流。

## 致谢

感谢 @ouuan 的 [冬日绘板模拟服务器](https://github.com/ouuan/fake-luogu-paintboard-server) 提供测试环境支持。

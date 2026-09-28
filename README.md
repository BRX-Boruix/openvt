# openvt

BORUIX 的终端管理工具：运行时申请一个新的终端实例。

[English](README.en.md)

## 用法

在 shell 中运行，无需参数：

```
[openvt] requesting a new console instance
[openvt] request queued for instance 4; wait for init patrol log (instance N created)
```

程序自己找一个空闲的实例号：从 1 向上尝试，跳过已在使用的号。递交请求后返回——终端的创建是
异步的，由系统初始化进程完成；确认可用的依据是初始化进程打印的创建日志。

## 行为约定

- 实例号上限 64，与系统创建终端的上限一致
- 开机预建的实例与已递交的请求都会被跳过
- 全部号码被占用时如实报「没有空闲的终端位」，以非零码退出
- 失败时打印错误类别（文件不存在、权限不足、已存在等），不打印笼统的失败

## 已知限制

- 不能指定实例号，号码由程序分配
- 请求递交后没有取消接口

## 构建

```bash
cargo build --release
```

## 文件结构

```
openvt/
├── Cargo.toml    # 包定义
├── build.rs      # 注入链接脚本
├── linker.ld     # 用户态段布局
└── src/
    └── main.rs   # 实例号试错与请求递交
```

## 相关项目

- [`init`](https://github.com/BRX-Boruix/init) —— 受理请求并创建终端
- [`consoled`](https://github.com/BRX-Boruix/consoled) —— 为新终端提供字节流
- [`login`](https://github.com/BRX-Boruix/login) —— 在新终端上提供登录

## 许可

MIT License，版权归 Yang Borui 所有。详见 [LICENSE](LICENSE)。

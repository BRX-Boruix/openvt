# openvt

**简体中文** | [English](#english)

在 BORUIX 上**打开一个新的终端**。

```
openvt
```

运行后，一个新的终端实例会被创建，你可以切换到它并独立登录。

---

## 它做什么

系统启动时会预先创建若干个终端实例。这些实例够用，但数量固定——`openvt` 让你在**运行时**再开
一个新的。

它本身不创建终端。它做的事情是**递交一个请求**：告诉系统初始化进程"我想要一个新终端"，然后就
返回。真正的创建由初始化进程在它的巡检循环中完成。

这个分工是有意的：终端实例的创建涉及启动多个守护进程、登记状态、分配资源，这些属于**系统级
编排**，应该由唯一的管理者统一处理，而不是让任意用户程序各自去创建。`openvt` 只是一个入口。

## 为什么用文件来递交请求

请求不是通过新增一个系统调用来表达的，而是通过**创建一个文件**。

这样做的好处来自文件系统本身已有的保证：**文件的原子创建天然提供了去重**。同名文件已经存在
就说明这个实例号已经被请求过了——不需要额外的锁或同步机制，竞争条件自然收敛。

请求文件位于 `/system/console-requests/` 目录下，文件名就是期望的实例号。初始化进程扫描这个
目录，看到合法请求就创建对应的终端，然后删掉请求文件表示"已受理"。

## 实例号是试出来的

用户**不指定**实例号（这是 `openvt` 这个工具的传统用法）——程序自己找一个空闲的号。

它在 1 往上依次尝试，遇到下面两种情况就跳到下一个号：

| 情况 | 判断方式 |
| --- | --- |
| 该号是开机预建的实例 | 设备节点已存在 |
| 该号已被别人请求 | 请求文件已存在 |

第一项是必要的：开机时预建的实例（编号从 0 开始）已经在使用中，不能再请求一次。

全部号码都被占用时，程序**如实报出"没有空闲的终端位"并以非零码退出**——而不是静默失败或无限
重试。上限是 64 个实例。

## 一个刻意的谨慎

程序在创建请求文件之前，会**先检查它是否已经存在**，而不是直接用"创建或清空"的方式打开。

原因是：如果请求文件已经存在，那说明**别人已经为这个号递交了请求**。用"创建或清空"打开会覆盖
掉那个待处理的请求——而真正的去重权威在初始化进程那边，请求方不应该擅自覆盖别人的请求。

所以这里的顺序是**先探测、再创建**：探测到已存在就退让，避免踩到别人的请求。

## 输出

成功递交请求后打印：

```
[openvt] request queued for instance N; wait for init patrol log (instance N created)
```

**请求递交成功不等于终端已经就绪**——真正的创建是异步的，由初始化进程在巡检时完成。确认终端
可用的依据是初始化进程打印的创建日志。

失败时程序打印错误**类别**（文件不存在、权限不足、已存在等），而不是笼统的"FATAL"。分类信息
让排查能立刻定位到是哪一环出了问题。

## 构建

```bash
cargo build --release
```

编译产物部署为 BORUIX 系统中的用户态程序，在 shell 中执行。

## 文件结构

```
openvt/
├── Cargo.toml    # 包定义
├── build.rs      # 注入链接脚本
├── linker.ld     # 用户态段布局
└── src/
    └── main.rs   # 程序本体
```

## 相关项目

- [`init`](https://github.com/BRX-Boruix/init) —— 受理请求并创建终端
- [`consoled`](https://github.com/BRX-Boruix/consoled) —— 为每个新终端提供字节流
- [`login`](https://github.com/BRX-Boruix/login) —— 在新终端上提供登录
- [`libsys`](https://github.com/BRX-Boruix/libsys) —— 用户态系统调用封装

## 许可

MIT License，版权归 Yang Borui 所有。详见 [LICENSE](LICENSE)。

---

# English

[简体中文](#openvt) | **English**

**Open a new terminal** on BORUIX.

```
openvt
```

After it runs, a new terminal instance is created, which you can switch to and log in on
independently.

---

## What it does

The system pre-creates a number of terminal instances at boot. That is enough to get going, but the
count is fixed — `openvt` lets you open one more **at run time**.

It does not itself create a terminal. What it does is **submit a request**: tell the system init
process "I would like a new terminal", and return. The actual creation happens in the init process's
patrol loop.

The division is deliberate: creating a terminal instance involves starting several daemons,
registering state, and allocating resources. That is **system-level orchestration** and belongs to a
single authority rather than to arbitrary user programs creating instances on their own. `openvt` is
merely the entry point.

## Why the request is a file

The request is not expressed by adding a system call, but by **creating a file**.

The benefit comes from a guarantee the filesystem already provides: **a file's atomic creation gives
deduplication for free**. The same-named file already existing means that instance number has already
been requested — no extra lock or synchronisation mechanism is needed, and the race collapses on its
own.

Request files live under `/system/console-requests/`, with the desired instance number as the file
name. The init process scans that directory, creates the matching terminal for a valid request, then
deletes the file to mark it consumed.

## The instance number is found by trying

The user **does not specify** the instance number (the traditional behaviour of this tool) — the
program finds a free one itself.

It tries upward from 1, skipping to the next number when either of these holds:

| Situation | How it is detected |
| --- | --- |
| The number is a pre-created boot instance | The device node already exists |
| The number has already been requested | The request file already exists |

The first is necessary: instances pre-created at boot (numbered from 0) are already in use and must
not be requested again.

When every number is taken, the program **honestly reports "no free console slot" and exits non-zero**
— rather than failing silently or retrying forever. The cap is 64 instances.

## One deliberate caution

Before creating the request file, the program **checks whether it already exists**, rather than
opening it in a create-or-truncate manner.

The reason: if the request file already exists, **someone else has already requested that number**.
Opening it create-or-truncate would overwrite that pending request — and the authoritative
deduplication lives with the init process, so a requester should not be overwriting other requests on
its own.

Hence the order here is **probe first, then create**: on finding it exists, back off rather than
treading on someone else's request.

## Output

After successfully submitting a request it prints:

```
[openvt] request queued for instance N; wait for init patrol log (instance N created)
```

**A submitted request does not mean the terminal is ready** — the actual creation is asynchronous,
performed by the init process on its next patrol. The evidence that a terminal is usable is the
creation log line printed by the init process.

On failure the program prints the error **category** (file not found, permission denied, already
exists, and so on) rather than a bare "FATAL". The classification lets a diagnosis land on the
failing step immediately.

## Building

```bash
cargo build --release
```

The artifact is deployed as a user-space program in a BORUIX system and run from the shell.

## Layout

```
openvt/
├── Cargo.toml    # package definition
├── build.rs      # injects the linker script
├── linker.ld     # user-space section layout
└── src/
    └── main.rs   # the program itself
```

## Related projects

- [`init`](https://github.com/BRX-Boruix/init) — accepts the request and creates the terminal
- [`consoled`](https://github.com/BRX-Boruix/consoled) — supplies the byte stream for each new terminal
- [`login`](https://github.com/BRX-Boruix/login) — provides login on the new terminal
- [`libsys`](https://github.com/BRX-Boruix/libsys) — the user-space syscall wrapper

## License

MIT License, copyright Yang Borui. See [LICENSE](LICENSE).

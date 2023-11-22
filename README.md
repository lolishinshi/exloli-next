# exloli-next

因为受不了当初乱写代码的自己而重写的新一代的 exloli 客户端

## 安装

### 通过 cargo

```bash
# 安装 rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# 激活 rust 环境
source $HOME/.cargo/env
# 安装 exloli-next
cargo install --git https://github.com/lolishinshi/exloli-next
# 测试是否安装成功
exloli-next --help
```

### 通过 docker

```bash
# 注：docker-compose 自行安装 
mkdir exloli-next && cd exloli-next
wget https://raw.githubusercontent.com/lolishinshi/exloli/master/docker-compose.yml
wget https://github.com/EhTagTranslation/Database/releases/download/v6.7880.1/db.text.json
touch db.sqlite db.sqlite-shm db.sqlite-wal
mv config.toml.example config.toml
docker-compose up -d
```

## 配置

请参考 config.toml.example

## 从 exloli 迁移

直接运行即可，但是建议备份好数据库

## TODO

- 处理旧本子的投票：通过 /query 返回 OR 重新编辑频道消息添加投票 OR ？
- 标记坏图片，可以分为无效图片和广告图片，后者不会上传，两者均不会出现在 challenge 中
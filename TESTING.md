# Making sure iced builds in all configurations

```shell
cargo build --features=applet,serde-keycode,smol
cargo build --features=applet,serde-keycode,tokio
```
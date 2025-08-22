# Making sure iced builds in all configurations

```shell
cargo build --features=applet,serde-keycode,winit,wgpu,smol
cargo build --features=applet,serde-keycode,winit,wgpu,tokio
cargo build --features=applet,serde-keycode,winit,wgpu,async-std
```
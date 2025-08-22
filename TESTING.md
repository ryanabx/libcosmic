# Making sure iced builds in all configurations

```shell
cargo build --features=applet,serde-keycode,winit,wgpu,smol
cargo build --features=applet,serde-keycode,winit,wgpu,tokio
cargo build --features=applet,serde-keycode,winit,wgpu,async-std
```

Rebase conflicted commits (in order):

- 9ac3318357d636cde22ae34f7b2cdeddd3f55cdb
- 659669dd5810d470d51f528495cab2f676eb58ed


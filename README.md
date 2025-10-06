# Websocket camera stream example


### Prerequisites
- Rust installed
- OpenCV installed (`brew install opencv` on macOS, `apt install libopencv-dev` on Ubuntu)
- `miniserve` installed for html stream preview (`cargo install miniserve`)



### Run

```
cargo run
```

### Run browser client for stream preview

```
miniserve ./web/ -p 8081
```

Then open `http://127.0.0.1:8081/index.html` in your browser.



Optional config.json (place next to executable):

```json
{
  "url": "127.0.0.1:8082",
  "frame_height": 540
}
```


### Build:


```
cargo build --release
```

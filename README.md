small, simple, and stupid osc visualizer for tidal

usage: `cargo run IP:port`

all the params + ranges are set in `src/dirt_display.rs`








ex:
`
cargo run -- --listen-addr 127.0.0.1:57120 \
  --param-display "delta:f32:raw" \
  --param-display "n:f32:custom_float" \
  --param-display "s:string:raw" \
  --param-display "orbit:i32:bar_int:0:9" \
  --param-display "gain:f32:bar_float:0.0:2.0" \
  --param-display "accelerate:f32:bar_float:-10.0:10.0" \
  --param-display "amp:f32:bar_float:0.0:2.0" \
  --param-display "pan:f32:bar_float:0.0:1.0" \
  --param-display "begin:f32:bar_float:0.0:1.0" \
  --param-display "end:f32:bar_float:0.0:1.0" \
  --param-display "speed:f32:bar_float:-10.0:10.0" \
  --param-display "release:f32:bar_float:0.0:4.0" \
  --param-display "cut:i32:bar_int:-1:9" \
  --param-display "cutoff:f32:bar_float:0.0:20000.0" \
  --param-display "hcutoff:f32:bar_float:0.0:20000.0" \
  --param-display "shape:f32:bar_float:0.0:1.0" \
  --param-display "coarse:f32:bar_float:0.0:5.0" \
  --param-display "distort:f32:bar_float:0.0:5.0" \
  --param-display "squiz:f32:bar_float:0.0:5.0" \
  --param-display "waveloss:f32:bar_float:0.0:100.0" \
  --param-display "delay:f32:bar_float:0.0:1.0" \
  --param-display "delaytime:f32:bar_float:0.0:1.0" \
  --param-display "delayfeedback:f32:bar_float:0.0:1.0" \
  --param-display "cycle:f32:cycle" # Don't forget cycle if you want its special display!
`
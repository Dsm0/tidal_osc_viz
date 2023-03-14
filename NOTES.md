currently uses a billion allocs, should just use the same string repeatedly

you ran into an alloc failure you dufus:
memory allocation of 6917529027641081853 bytes failed  :1   
[1]    11067 IOT instruction (core dumped)  cargo run 127.0.0.1:5223

you should definitely be passing around &str


TODO: refactor to minimize memory usage/allocations,
should definitely do before you build out the app
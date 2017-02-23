# Rest Api Client

## Clients

```
rac pocket auth
rac pocket list
rac pocket rm <id>,<id>

rac centerdevice status
rac centerdevice status details

rac centerdevice auth
rac centerdevice upload <file>
rac centerdevice search <search>
rac centerdevice download <document-id>

rac elasticsearch status -- curl -s http://es-m-05:9200/_cluster/health?level=shards
rac elasticsearch health -- curl -s http://es-m-05:9200/_cluster/health
```


## Todos

* [ ] Fill out Cargo.toml

* [ ] Clap bash and zsh autocompletion


## References

* [Rust and Rest](http://lucumr.pocoo.org/2016/7/10/rust-rest/)

* [Starting a new Rust project right, with error-chain](https://brson.github.io/2016/11/30/starting-with-error-chain)


## Crates

* [curl-rust](https://github.com/alexcrichton/curl-rust/commits/master)

* [oauth2](https://github.com/alexcrichton/oauth2-rs/blob/master/src/lib.rs)

* [serde](https://serde.rs)


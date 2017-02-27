# Rest Api Tool

## Clients

```
rat pocket auth
rat pocket list
rat pocket rm <id>,<id>

rat centerdevice status
rat centerdevice status details

rat centerdevice auth
rat centerdevice upload <file>
rat centerdevice search <search>
rat centerdevice download <document-id>

rat elasticsearch status -- curl -s http://es-m-05:9200/_cluster/health?level=shards
rat elasticsearch health -- curl -s http://es-m-05:9200/_cluster/health

rat slack send <channel> <message>
```


## Todos

* [X] Clap

    * [X] bash and zsh autocompletion

* [ ] Fill out Cargo.toml

* [ ] Error Chain

* [ ] Brew and deb


## References

### Pocket

* [Pocket Auth](http://www.jamesfmackenzie.com/getting-started-with-the-pocket-developer-api/)

### Rust

* [Rust and Rest](http://lucumr.pocoo.org/2016/7/10/rust-rest/)

* [Starting a new Rust project right, with error-chain](https://brson.github.io/2016/11/30/starting-with-error-chain)


## Crates

* [curl-rust](https://github.com/alexcrichton/curl-rust/commits/master)

* [oauth2](https://github.com/alexcrichton/oauth2-rs/blob/master/src/lib.rs)

* [serde](https://serde.rs)


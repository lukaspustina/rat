# Rest Api Tool

## Available Clients

```
rat pocket auth
rat pocket list

rat centerdevice status [--details]
```

## Installation

### macOS with Homebrew

```
brew install lukaspustina/os/rat
```

----

## Clients to Come

```
rat pocket rm --ids <id>,<id>

rat centerdevice browse-status
rat centerdevice auth
rat centerdevice upload <file>
rat centerdevice search <search>
rat centerdevice download <document-id>

rat elasticsearch browse
rat elasticsearch status -- curl -s http://es-m-05:9200/_cluster/health?level=shards
rat elasticsearch health -- curl -s http://es-m-05:9200/_cluster/health

rat slack send <channel> <message>

rat bosun browse
rat bosun show-incidents --open
rat bosun close-incidents --ids  <id>,<id> --message "Just because ..."
```


## Todos

* [X] Clap

    * [X] bash and zsh autocompletion

* [X] Fill out Cargo.toml

* [X] Error Chain

* [X] Distributions

    * [X] Brew

    * [X] Debian

* [ ] Travis

    * [ ] packagecloud.io

* [ ] Tests

    * [ ] Add Badges for Travis to Cargo.toml


----

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


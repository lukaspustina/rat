# Rest Api Tool

## Available Clients

```bash
rat pocket auth
rat pocket list [--details] --tags <tag>,<tag> [--state unread|archive|all] [--sort newest|oldest|title|site ]
rat pocket search [--details] --tags <tag>,<tag> [--state unread|archive|all] [--sort newest|oldest|title|site ] <in title or url>
rat pocket archive|readd|favorite|unfavorite|delete <id>...

rat centerdevice status [--details]
rat centerdevice browse-status [--details]
```

## Installation

### macOS with Homebrew

```
brew install lukaspustina/os/rat
```


## Usage

### Pocket

#### Authentication

##### Create a consumer key

Create a [new application](https://getpocket.com/developer/apps/new) and add the newly created consumer key to your rat configuration, section [pocket] as `consumer_key = '<consumer key'`.

##### Authenticate rat

`rat pocket auth` and follow the instructions.

#### Archive| Readd| Favorite| Unfavorite | Delete

`rat pocket archive|readd|favorite|unfavorite|delete <article ids>...`

#### List

##### Filter articles for ids and titles

`rat -o json pocket list | jq '.list | .[] | { title: .given_title, id: .item_id }'`

##### Filter articles for ids and titles and search for Rust in title and URL

* `rat -o json pocket search Rust | jq -r '.list | .[] | { title: .given_title, id: .item_id }'`

* `rat -o json pocket list | jq '.list | .[] | { title: .given_title, id: .item_id, url: .given_url } | select((.title | test("Rust")) or (.url | test("Rust")))'`

##### Filter ids, search for Rust in title and URL, make comma seperated list

`rat -o json pocket search Rust | jq -r '.list | .[] | .item_id' | paste -s -d , -`


### Slack

#### Authentication

##### Create client ID and client Secret

Create a [new application](https://api.slack.com/apps) and add the newly created `Client ID` and `Client Secret` to your rat configuration, section [slack] as `client_id = '<client id>'` and `client_secret = '<client secret'`, respectively.

##### Authenticate rat

`rat slack auth` and follow the instructions.

----

## Clients to Come

```
rat centerdevice auth
rat centerdevice upload <file>
rat centerdevice search <search>
rat centerdevice download <document-id>

rat elasticsearch browse
rat elasticsearch status -- curl -s http://es-m-05:9200/_cluster/health?level=shards
rat elasticsearch health -- curl -s http://es-m-05:9200/_cluster/health

rat slack channel list
rat slack user list
rat slack send <channel|user> <message>

rat bosun browse
rat bosun incidents show --open
rat bosun incidents ack|close --ids  <id>,<id> --message "Just because ..."
rat bosun show-silences
rat bosun silence show
rat bosun silence set
```


## Todos

* [X] Clap

    * [X] bash and zsh autocompletion

* [X] Fill out Cargo.toml

* [X] Error Chain

* [X] Distributions

    * [X] Brew

    * [X] Debian

* [X] Finish Pocket

* [X] Output

    * [X] General option

    * [X] colors: regular, info(blue), warnings(yellow), error(red)

    * [X] add info msgs to modules

    * [x] quiet option

    * [X] Add output format parameter

    * [X] Apply output format to all, well, outputs

* [ ] Slack Auth

* [ ] CenterDevice Auth

* [ ] Move Strings to real ErrorKinds for Curl, etc; check error_chain for links to rmove "Curl failed" string Errors.

* [ ] Enhance Pocket

    * [ ] Add --since and --until parameters for search

    * [ ] Select human outout fields: id, title, url

    * [ ] Update documentation

* [ ] Travis

    * [ ] packagecloud.io

    * [ ] Add Badges for Travis to Cargo.toml

* [ ] Tests


----

## References

### Pocket

* [Pocket API](https://getpocket.com/developer/)

* [Pocket Auth](http://www.jamesfmackenzie.com/getting-started-with-the-pocket-developer-api/)

### Rust

* [Rust and Rest](http://lucumr.pocoo.org/2016/7/10/rust-rest/)

* [Starting a new Rust project right, with error-chain](https://brson.github.io/2016/11/30/starting-with-error-chain)


## Crates

* [curl-rust](https://github.com/alexcrichton/curl-rust/commits/master)

* [oauth2](https://github.com/alexcrichton/oauth2-rs/blob/master/src/lib.rs)

* [serde](https://serde.rs)


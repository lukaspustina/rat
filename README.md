# RAT - the Rest Api Tool

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Table of Contents**

- [Available Clients](#available-clients)
- [Installation](#installation)
  - [macOS with Homebrew](#macos-with-homebrew)
- [Usage](#usage)
  - [General](#general)
  - [CenterDevice](#centerdevice)
      - [Authenticate rat](#authenticate-rat)
      - [Download document](#download-document)
      - [Search document](#search-document)
      - [Upload document](#upload-document)
  - [Pocket](#pocket)
    - [Authentication](#authentication)
      - [Create a consumer key](#create-a-consumer-key)
      - [Authenticate rat](#authenticate-rat-1)
    - [Archive | Readd | Favorite | Unfavorite | Delete](#archive--readd--favorite--unfavorite--delete)
    - [List](#list)
      - [Advanced listing](#advanced-listing)
  - [Slack](#slack)
    - [Authentication](#authentication-1)
      - [Create client ID and client Secret](#create-client-id-and-client-secret)
      - [Authenticate rat](#authenticate-rat-2)
  - [Stocks](#stocks)
    - [Scrape current stock price from comdirect web page](#scrape-current-stock-price-from-comdirect-web-page)
- [Clients to Come](#clients-to-come)
- [Todos](#todos)
- [References](#references)
  - [Pocket](#pocket-1)
  - [Rust](#rust)
- [Crates](#crates)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

# Available Clients

**CenterDevice**
```bash
rat centerdevice auth
rat centerdevice download
rat centerdevice search
rat centerdevice status
rat centerdevice upload
```

**Pocket**
```bash
rat pocket auth
rat pocket list
rat pocket search
rat pocket archive
rat pocket readd
rat pocket favorite
rat pocket unfavorite
rat pocket delete
```

**Slack**
```bash
rat slack auth
```

**Stocks**
```bash
rat stocks
```

# Installation

## macOS with Homebrew

```bash
brew install lukaspustina/os/rat
```


# Usage

## General

There are two response output modes, i.e., JSON and HUMAN, and three message levels, i.e., QUIET, NORMAL, and VERBOSE.

In human output mode tries to give a concise representation if the received information. In json output mode, rat tries to pass the whole json response to the user -- if any is available. The output mode can be selected with the parameter `--output <humen|json>`, e.g., `rat --output json ...`

The message output levels configure how talkative rat is during execution. The quiet level reduces outputs to real responses only while the normal level tries to give some feedback to the user about the start and end of a request. The verbose level goes one step further and tries to inform the user about each step of the processing. The message level can be chosen with the parameters `--quiet` or `--verbose`, respectively. If none of these switches is activated, the message level is set to normal. For example, `rat --verbose ...` selects the verbose message level.

## CenterDevice

### Authenticate rat

`rat centerdevice auth` and follow the instructions.

### Download document

* Download document into current directory with original filename: `rat centerdevice download excbd68a-c397-id46-9350-a4fd4022fe8c`

* Download document with new filename: `rat centerdevice download excbd68a-c397-id46-9350-a4fd4022fe8c -f new_filename`

### Search document

* Search for documents with filename _README.md_, tagged with _documentation_, and some fulltext: `rat centerdevice search -f README.md -t documentation -t rat "the Rest Api Tool"`

* Search again, but now post-process JSON response with `jq`: `rat --output json --quiet centerdevice search -f README.md -t documentation "the Rest Api Tool" | jq .`

### Upload document

`rat centerdevice upload README.md -t documentation -t rat`


## Pocket

### Authentication

#### Create a consumer key

Create a [new application](https://getpocket.com/developer/apps/new) and add the newly created consumer key to your rat configuration, section [pocket] as `consumer_key = '<consumer key'`.

#### Authenticate rat

`rat pocket auth` and follow the instructions.

### Archive | Readd | Favorite | Unfavorite | Delete

`rat pocket archive|readd|favorite|unfavorite|delete <article ids>...`

### List

List all

* unread articeles: `rat pocket list`

* archived articeles: `rat pocket list --state archived`

* articles: `rat pocket list --state all`

* unread articles tagged with _Rust_: `rat pocket list --tag Rust`

* unread articles, added between 2 weeks and 1 week ago: `rat pocket list --since 2w --until 1w`

* List ids of all unread articles added 2 weeks or later ago and create a comma separated list: `rat pocket list --until 2w --output id | paste -s -d . -`

#### Advanced listing

* List all unread articles that contain a video: `rat --output json --quiet pocket list | jq '.list | .[] | select(.has_video | test("1") ) | { id: .item_id, title: .resolved_title }'`

* Filter articles that contain Rust in title and URL, and create comma separated id list: `rat -o json --quiet pocket list | jq -r '.list | .[] | { title: .given_title, id: .item_id, url: .given_url } | select((.title | test("Rust")) or (.url | test("Rust"))) | .id' | paste -s -d , -`


## Slack

### Authentication

#### Create client ID and client Secret

Create a [new application](https://api.slack.com/apps) and add the newly created `Client ID` and `Client Secret` to your rat configuration, section [slack] as `client_id = '<client id>'` and `client_secret = '<client secret'`, respectively.

#### Authenticate rat

`rat slack auth` and follow the instructions.


## Stocks

### Scrape current stock price from comdirect web page

* by company name: `rat stocks "Comdirect"`

* by WKN: `rat stocks 542800"`


----

# Clients to Come

```bash
rat elasticsearch browse
rat elasticsearch status -- curl -s http://<host>:9200/_cluster/health?level=shards
rat elasticsearch health -- curl -s http://<host>:9200/_cluster/health

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


# Todos

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

* [X] Slack Auth

* [X] CenterDevice Auth

* [X] CenterDevice Upload

    * [X] Use streams

* [X] Checkout mime_multipart 0.5 with my patch included

* [X] Move centerdevice browse-status to status --browse

* [X] Enchance Auths by --browser which opens a browser window.

* [X] Enhance Pocket

    * [X] Add --since and --until parameters for search

    * [X] Select human outout fields: id, title, url

    * [X] Update documentation

* [X] Refactor auth modules - cf. branch

* [X] Slack: Move HTTP calls to client mod

* Ongoing

    * [ ] Replace curl with hyper

    * [ ] Pocket: Move HTTP calls to client mod

* [ ] Make better use of error_chain by using Foreign errors

* [ ] Travis

    * [ ] packagecloud.io

    * [ ] Add Badges for Travis to Cargo.toml

* [ ] Tests

* [ ] Run Clippy once cargo in nightly works again -- cf. http://asquera.de/blog/2017-03-03/setting-up-a-rust-devenv/

* [ ] Move to future based clients -- cf. http://asquera.de/blog/2017-03-01/the-future-with-futures/


----

# References

## Pocket

* [Pocket API](https://getpocket.com/developer/)

* [Pocket Auth](http://www.jamesfmackenzie.com/getting-started-with-the-pocket-developer-api/)

## Rust

* [Rust and Rest](http://lucumr.pocoo.org/2016/7/10/rust-rest/)

* [Starting a new Rust project right, with error-chain](https://brson.github.io/2016/11/30/starting-with-error-chain)


# Crates

* [curl-rust](https://github.com/alexcrichton/curl-rust/commits/master)

* [oauth2](https://github.com/alexcrichton/oauth2-rs/blob/master/src/lib.rs)

* [serde](https://serde.rs)

* Testing

    * [Discussion of frameworks](https://www.reddit.com/r/rust/comments/5jbezo/testing_frameworks/)

    * Common Pattern for test setups, also see [this](https://medium.com/@ericdreichert/test-setup-and-teardown-in-rust-without-a-framework-ba32d97aa5ab#.yfl5o576u)
    ```Rust
    #[test]
    fn make_sure_foo_works() {
        setup(|&fixture_state| { ... });
    }
    ```

    * BDD

        * [Stainless](https://github.com/reem/stainless)

        * [rspec](https://github.com/mackwic/rspec)

    * [spectral](https://github.com/cfrancia/spectral)

    * [Quickcheck](https://github.com/BurntSushi/quickcheck)

    * [Mocks](https://github.com/kriomant/mockers)


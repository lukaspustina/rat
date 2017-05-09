![Rat](images/rat.png)

## RAT - the Rest Api Tool

Rat is a simple command line tool that helps me automate repetitive tasks for my favorite web services and software products which expose a REST API. The general guideline for all commands rat supports is that they should perform only one thing preferably with one HTTP call only. This guideline follows the UNIX principle that tools should to one thing and complex behaviour is achieved by pipelining.

[![Linux & OS X Build Status](https://img.shields.io/travis/lukaspustina/rat.svg?label=Linux%20%26%20OS%20X%20Build%20Status)](https://travis-ci.org/lukaspustina/rat) [![Windows Build status](https://img.shields.io/appveyor/ci/lukaspustina/rat.svg?label=Windows%20Build%20Status)](https://ci.appveyor.com/project/lukaspustina/rat/branch/master) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg?label=License)](./LICENSE) [![](http://meritbadge.herokuapp.com/rat)](https://crates.io/crates/rat)

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Table of Contents**

- [Available Clients](#available-clients)
- [Installation](#installation)
  - [macOS](#macos)
  - [Ubuntu](#ubuntu)
  - [Windows](#windows)
  - [From Source](#from-source)
- [Usage](#usage)
  - [General](#general)
  - [CenterDevice](#centerdevice)
    - [Authenticate rat](#authenticate-rat)
    - [Refresh access token](#refresh-access-token)
    - [Search collection](#search-collection)
    - [Download document](#download-document)
    - [Search document](#search-document)
    - [Upload document](#upload-document)
    - [Delete document](#delete-document)
  - [Pocket](#pocket)
    - [Authentication](#authentication)
      - [Create a consumer key](#create-a-consumer-key)
      - [Authenticate rat](#authenticate-rat-1)
    - [Archive | Readd | Favorite | Unfavorite | Delete](#archive--readd--favorite--unfavorite--delete)
    - [List and Search](#list-and-search)
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
- [Credits](#credits)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

# Available Clients

**CenterDevice**
```bash
rat centerdevice auth
rat centerdevice collections
rat centerdevice delete
rat centerdevice download
rat centerdevice refresh_token
rat centerdevice search
rat centerdevice status
rat centerdevice upload
```

**Pocket**
```bash
rat pocket auth
rat pocket list
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

## macOS

Please use [Homebrew](https://brew.sh) to install rat on your system.

```bash
brew install lukaspustina/os/rat
```

## Ubuntu

Pleae add my [PackageCloud] open source repository and install rat via apt.

```bash
curl -s https://packagecloud.io/install/repositories/lukaspustina/opensource/script.deb.sh | sudo bash
sudo apt-get install rat
```

## Windows

rat is automatically build on Windows via AppVeyor to ensure is compiles and runs fine but binaries are currently not provided. Please compile it from source.

## From Source

Please install Rust via [rustup](https://www.rustup.rs) and then run

```bash
cargo install rat
```


# Usage

## General

There are two response output modes, i.e., JSON and HUMAN, and three message levels, i.e., QUIET, NORMAL, and VERBOSE.

In human output mode tries to give a concise representation if the received information. In json output mode, rat tries to pass the whole json response to the user -- if any is available. The output mode can be selected with the parameter `--output <humen|json>`, e.g., `rat --output json ...`

The message output levels configure how talkative rat is during execution. The quiet level reduces outputs to real responses only while the normal level tries to give some feedback to the user about the start and end of a request. The verbose level goes one step further and tries to inform the user about each step of the processing. The message level can be chosen with the parameters `--quiet` or `--verbose`, respectively. If none of these switches is activated, the message level is set to normal. For example, `rat --verbose ...` selects the verbose message level.

## CenterDevice

### Authenticate rat

`rat centerdevice auth` and follow the instructions.

### Refresh access token

`rat centerdevice refresh_token`

### Search collection

* Show my collections: `rat centerdevice collections`

* Show my as well as public collections: `rat centerdevice collections --public-collections`

* Search collections: `rat centerdevice collections <search filter>`

* Cache collection names for other commands: `rat centerdevice collections --cache`

### Download document

* Download document into current directory with original filename: `rat centerdevice download excbd68a-c397-id46-9350-a4fd4022fe8c`

* Download document with new filename: `rat centerdevice download excbd68a-c397-id46-9350-a4fd4022fe8c -f new_filename`

### Search document

* Search for documents with filename _README.md_, tagged with _documentation_, and some fulltext: `rat centerdevice search -f README.md -t documentation -t rat "the Rest Api Tool"`

* Search again, but now post-process JSON response with `jq`: `rat --output json --quiet centerdevice search -f README.md -t documentation "the Rest Api Tool" | jq .`

* Expand search to public collections: `... --public-collections`

### Upload document

* Upload documen with tags: `rat centerdevice upload README.md -t documentation -t rat`

* Upload document to collection: `rat centerdevice upload README.md --collection <collection id>`

* Upload document to collection using collection cache: `rat centerdevice upload README.md --Collection <cached collection name>`

### Delete document

`rat centerdevice delete excbd68a-c397-id46-9350-a4fd4022fe8c`


## Pocket

### Authentication

#### Create a consumer key

Create a [new application](https://getpocket.com/developer/apps/new) and add the newly created consumer key to your rat configuration, section [pocket] as `consumer_key = '<consumer key'`.

#### Authenticate rat

`rat pocket auth` and follow the instructions.

### Archive | Readd | Favorite | Unfavorite | Delete

`rat pocket archive|readd|favorite|unfavorite|delete <article ids>...`

### List and Search

Search in title and URL of all articles ...

* the word rust: `rat pocket list rust`

List all ...

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
rat feedly

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

* [X] Replace curl with hyper

* [X] Pocket: Move HTTP calls to client mod

* [X] Replace Pocket list and search by search that optionally takes a search term

* [X] Travis

    * [X] packagecloud.io

    * [X] Add Badges for Travis in Cargo.toml and Readme

* [X] Wait for new select release and then let Travis publish to crates.io

* [X] CenterDevice

    * [X] Refactor client lib

    * [X] search for collections

    * [X] upload to collection

    * [X] named collections

    * [X] upload to named collections

    * [X] delete document

    * [X] Token Refresh / Error messages when token has expired

* [X] Elastic tabstops for output where appropriate

    * [X] pocket list -- but not everywhere

* [ ] Tests

    * [X] Setup

        * [X] clippy

        * [X] docker run

    * [ ] Select integration test framework

    * [ ] Integration tests

    * [ ] Unit tests with quick check, naught strings

    * [ ] Fuzzing tests

    * [ ] [kcov with Docker](http://sunjay.ca/2016/07/25/rust-code-coverage)

* [ ] Make better use of error_chain

    * [ ] by using Foreign errors

    * [ ] by using [ensure!](https://docs.rs/error-chain/0.10.0/error_chain/macro.ensure.html)

* [ ] Move to future based clients -- cf. http://asquera.de/blog/2017-03-01/the-future-with-futures/

-- Before first 1.0 release

* [ ] Documentation

    * [ ] Add documentation generation to .travis

* [ ] Cleanup of Readme; esp. Todos

----

# References

## Pocket

* [Pocket API](https://getpocket.com/developer/)

* [Pocket Auth](http://www.jamesfmackenzie.com/getting-started-with-the-pocket-developer-api/)

## Rust

* [Rust and Rest](http://lucumr.pocoo.org/2016/7/10/rust-rest/)

* [Starting a new Rust project right, with error-chain](https://brson.github.io/2016/11/30/starting-with-error-chain)


# Crates

* [elastic tabstops](https://github.com/BurntSushi/tabwriter)

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

# Credits

* Rat icon is (C) by [Eduardo Gonçalves Costa](https://thenounproject.com/geceduardo) under [CC BY 3.0 US](https://creativecommons.org/licenses/by/3.0/us/).


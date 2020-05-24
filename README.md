Task Board
==========

Simple & extremly fast Taskwarrior web frontend


## Rationale

There are seveval third party Taskwarrior web interfaces, but they fell into one of these categories:

* They feel extremely bloated. Installing a docker container for a frontend to Taskwarrior feels in total opposite to the spirit of the original tool. Most importantly the Taskwarrior data model is simple and elegant, and I believe it can match nicely with its visual representation: task attributes each have simple types and can be represented by HTML elements to mimic the command line UI. There is no need for that latest fancy JavaScript framework BS, less is more.
* They are commercial applications. I have nothing against paying for good software, but hosting my tasks lists unencrypted on an third party server is a big no no for me.


## Goals

### Usability

* be extremely fast, **any perceived latency is a bug**
* be easily usable on mobile (this is a priority target compared to desktop)
* support the whole `task` workflow: add, delete, edit, undo, etc. (no less, no more)

I *dogfood* and use this everyday, so you bet this is important to me.

### Technical goals

* application is a single binary, with everything (including assets) in it
* do one thing, and do it well: get configuration from the usual `~/.taskrc` and offer a convenient UI to interact with tasks though the `task` binary, everything else is out of scope
* minimalist presentation layer for simplicity and performance:
    * use plain HTML as much as possible, with semantic elements
    * use JavaScript only when absolutely necessary (ideally under 10 lines of code, to trigger updates)
    * minimal use of CSS, with a ["drop in" stylesheet](https://github.com/dohliam/dropin-minimal-css#list-of-frameworks)
    * use font icons, instead of images
* improve my Rust skills (which tend to get rusty these days :smirk:)


## Progress

### Done

**Nothing**

### TODO

**Everything minus what is done**


## Install

*TODO*


## Deployement

*TODO reverse proxy*


## License

*TODO*

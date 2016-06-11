# httpbox

[![Build Status](https://travis-ci.org/kevinastone/httpbox.svg?branch=master)](https://travis-ci.org/kevinastone/httpbox)

httpbox is an HTTP test tool that provides a number of endpoints for testing a
variety of HTTP features similar to [httpbin](http://httpbin.org).  It is
written in [Rust](https://www.rust-lang.org) and uses the [Iron web framework](http://ironframework.io).

You can see it in action [here](http://whispering-shelf-71295.herokuapp.com).


## Installation

[![Deploy](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy)

    git clone https://github.com/kevinastone/httpbox.git
    cd httpbox
    cargo run
    open http://localhost:3000

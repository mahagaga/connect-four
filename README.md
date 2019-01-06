# connect-four
The goal is to have a decent Connect Four player implemented in Rust.

### Crates
There are two crates

- ***game***: core game player.
- ***server***: a primitive web server meant to run on localhost serving the graphical user interface,
 e.g. a frontend web server, with JSON type data.

### Installation instructions

* clone this reopository and connect-four-js, forked from bryanbraun/connect-four for the frontend

```
git clone https://github.com/scem/connect-four.git
git clone https://github.com/scem/connect-four-js.git
```

* then you need rust installed
if not and you're on Ubuntu you can just call:

```
curl https://sh.rustup.rs -sSf | sh
```

* now you're ready for running the backend service

```
cd connect-four/server
cargo run
```

* you need a webserver running on your computer.
if that's not the case and you're on Ubuntu you can just call:

```
sudo apt install apache2
```

* make the frontend visible in your webserver.
for apache and Ubuntu call:

```
ln -s `readlink -f connect-four-js` /var/www/html/connect-four
```

* done! you can play on 'http://localhost/connect-four'.
have fun!

### License
This project is licensed under GNU General Public License v3.0

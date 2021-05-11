vimdir
======

A clone of the handy vidir(1) utility that is part of the moreutils
package [1]. Essentially it allows you to edit the names of files and
directories with your text editor.

Current State
-------------

It is called vimdir because at some point it wants to be just like
vidir, but improved.

As of now, it might just randomly delete your $HOME. Run at your own
risk. It is also the first real Rust program I have written. So if I
am not using idiomatic Rust, feel free to correct me.

Documentation
-------------

The /doc directory contains some documentation. It uses markdown as
interpreted by ronn(1) [2] which can be used to generate a manpage.

Credit
------

I am not a lawyer, but to be totally safe I consider this project a
derivative work of the original vidir(1). In particular, parts of the
documentation in /doc is copied from the moreutils project. As such
this project is licensed under the GPL2 just like moreutils is.

References
----------

[1] https://github.com/madx/moreutils
[2] https://rtomayko.github.io/ronn/

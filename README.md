# AMF Viewer

Wireshark is good and all but:

1) It's not Rust
2) It's not mine
3) It doesn't do like one thing that I would like
4) It's missing like one tiny thing that ***may*** help debugging

So I'm making a "simple" rust application to visualise AMF/RTMP packets, and hopefully it'll be
enough to debug issues with my main project. Mainly though, this was done just to scratch my
Rust itch until I progress further in my main project to start writing a Rust XMPP server.

# Running

These sections are here for later me because I can barely remember the damn package's name.

`dx serve --platform desktop --hot-reload=false`

# Bundle

Fairly certain I need to add more to the code to actually allow for this to happen?

`dx bundle --release`
# Blogger

A more or less pointless blogging language that compiles from a blogging DSL straight to HTML with tailwind styling.

The compiler is written with no dependencies. The only dependency for the project is the WASM generation tooling. Everything, from the regex matcher, the lexer, parser and compiler backend is written from scratch.

Even the crude CLI interface is made from scratch.

I'm not a Rust developer. I chose Rust because I was interested in it. If you stumble upon this repo and have constructive criticism as to how it could be better, please do leave an issue or send me a message.

I use this to write my blogs. I upload the blog files to object storage and retrieve them in an Astro application, using the WASM bindings to compile the blog into HTML to render.


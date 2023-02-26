# Showdown Poll - Rust + Leptos
This is a prototype for a leptos embed that aims to bring randomized polls of Pokemon move sets provided by the [smogon strategy dex](https://www.smogon.com/dex/sv/pokemon/). Currently it only pulls data from the Scarlet and Violet generation for minimal resource consumption during development. Other generations will be added in the future.

The idea of this was inpsired by [@imablisy](https://www.youtube.com/@imablisy) and can be used to find new Pokémon to build and put together teams with the help of chat.

## Building
Currently, this repo requires a dev build of `cargo-leptos` found [here](https://github.com/SteveCookTU/cargo-leptos/tree/pr/fix-wasm-imports). This branch is currently for a PR to fix imports when binding to JavaScript through `wasm_bindgen`.

## Caveats
Without the ability to access/study orgs and permissions, this package does not use proper permissions and each server instance can only be used for one streamer at a time. Unfortunately, the streamer's user ID must also be hard-coded into the compilation of the server until an API key is available to properly implement these features. Until then, this project is will stay in prototype phase.

## Truffle Dev Extension Import
The initial script used to import this embed and used with the extension is as follows:

    localStorage.setItem(
      "truffle:devExtensionMappings",
      JSON.stringify([
        {
          iframeUrl: "http://127.0.0.1:3000/",
          domAction: null,
          defaultLayoutConfigSteps: [
            { action: "querySelector", value: "body" },
            { action: "appendSubject", value: null },
            { action: "useSubject", value: null },
            {
              action: "setStyle",
              value: {
                position: "fixed",
                width: "100px",
                height: "40px",
    			border-top-left-radius: "0.375rem",
    			border-top-right-radius: "0.375rem",
                bottom: 0,
                "z-index": 1000,
              },
            },
          ],
        },
      ])
    );

Changes made in the `[package.metadata.leptos]` of `Cargo.toml` regarding `site-addr` must be reflected in the `iframeUrl` field.

## Demonstration
The following image will redirect to a YouTube video that demonstrates the functionality of the embed.

[![Embed Demonstration](https://img.youtube.com/vi/PMm2upotHRQ/0.jpg)](https://www.youtube.com/watch?v=PMm2upotHRQ)


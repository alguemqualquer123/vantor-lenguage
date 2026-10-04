# Lexicon — Get Started

Welcome to Lexicon! This guide takes you from zero to a running HTTP API in four steps.

## 1. Install the `lex` toolchain

Make sure the `lex` binary is on your `PATH`, then verify it:

```sh
lex install
lex --version
```

> If `lex` is not found, set the **Lexicon: Path** setting (`lexicon.path`)
> to the binary location and re-run the command above in a terminal.

## 2. Create a project

[Create a new Lexicon project](command:lexicon.newProject)

Pick a project name (for example `my_lex_app`). The command scaffolds a
project with a `main.lex` entry point plus `.env.dev` / `.env.prod` files.

## 3. Run the demo API with watch mode

Open the scaffolded folder and start the development server:

```sh
lex run --watch
```

The server reloads on every save. Try editing a `@Get` handler in
`main.lex`, save, and refresh `http://localhost:3000` in your browser.

## 4. Run tests

Lexicon picks up every `@Test`-decorated function. Run the suite with:

```sh
lex test
```

…or open the Command Palette and choose **Lexicon: Run Tests**.
Green output means your handlers, JSON parsing and DB queries all pass —
you are ready to build something real. Happy hacking!

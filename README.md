[![rust](https://img.shields.io/badge/rust-161923?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![git-logo](https://img.shields.io/badge/git-F05032?style=for-the-badge&logo=git&logoColor=white)](https://git-scm.com/)

[![license](https://img.shields.io/github/license/xsv24/git-kit?color=blue&style=flat-square&logo=)](./LICENSE)

# 🧰 git-kit

cli to help format your git commit messages consistent with less effort via pre-provided templates 🤩.

```bash
git-kit --help
```

## 🥽 Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)

## 🏎️💨 Getting Started

```bash
git-kit checkout TICKET-123
```

```bash
git-kit commit bug -m "fix"
```
> This will create an editable commit with the following format and will insert branch name will be injected by default into the `bug` commit template.
>
> `[TICKET-123] 🐛 fix`


## ⚙️ Settings 

```bash
git-kit --help
```

## 🎮 Overriding 

Planning to provide a way to allow your own templates at a global or repository level.

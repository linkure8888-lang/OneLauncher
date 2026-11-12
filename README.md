# OneClient

[![Build and Release](https://github.com/linkure8888-lang/OneLauncher/actions/workflows/build.yml/badge.svg)](https://github.com/linkure8888-lang/OneLauncher/actions/workflows/build.yml)

Современный кроссплатформенный лаунчер для Minecraft (написан на Rust + Freya) и собственный набор Fabric-модов-компаньонов.

## Сборка и разработка

Подробные инструкции по сборке, требования к окружению и решению возможных проблем с кэшем собраны в [CONTRIBUTING.md](OneLauncher/companion/CONTRIBUTING.md).

### Быстрый старт
- **Лаунчер (Rust):**
  ```bash
  cd OneLauncher
  cargo build --release
  ```
- **Мод-компаньон (Java 25 / Fabric):**
  ```bash
  cd OneLauncher/companion
  ./gradlew build
  ```

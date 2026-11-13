# Руководство контрибьютора OneClient / Companion

Добро пожаловать в проект OneClient! Это руководство поможет вам успешно настроить окружение и собрать проект без ошибок.

## Если сборка падает непонятно почему
Если Gradle выдает странные ошибки кэширования Loom или миксинов, выполните полную очистку:
```bash
./gradlew cleanloom && ./gradlew clean && ./gradlew build --no-daemon
```

## Требования для Linux
Для успешной компиляции лаунчера на Rust и мода на Linux требуются следующие системные библиотеки:
- `libvulkan-dev`
- `libx11-dev`
- `libxcb1-dev`
- `libxkbcommon-dev`
- `libwayland-dev`
- `libssl-dev`
- `pkg-config`

## Требования для macOS
Убедитесь, что установлены Xcode Command Line Tools:
```bash
xcode-select --install
```

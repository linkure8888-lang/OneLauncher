# telegram-post.ps1 — публикация в Telegram

## Быстрый старт

1. Откройте Telegram, найдите **@BotFather**, отправьте `/newbot`, следуйте инструкциям.
   BotFather выдаст токен вида `123456:ABC-DEF…`.

2. Добавьте бота **админом** в канал `@remixclient_ru` (или ваш канал).
   Без прав админа бот не сможет публиковать.

3. Сохраните токен в файл (одна строка, без пробелов):
   ```
   mkdir %USERPROFILE%\.remix
   echo 123456:ABC-DEF… > %USERPROFILE%\.remix\tg-token.txt
   ```
   Либо задайте переменную окружения `REMIX_TG_TOKEN`.

4. Проверьте через dry run (ничего не отправляет):
   ```powershell
   .\tools\telegram-post.ps1 -Text "Привет, мир!" -DryRun
   ```

5. Отправьте в канал:
   ```powershell
   .\tools\telegram-post.ps1 -Text "<b>RemixClient 0.2</b> — новая версия!"
   .\tools\telegram-post.ps1 -File .\changelog.md -Silent
   ```

## Параметры

| Параметр | Описание |
|---|---|
| `-Text "..."` | Текст сообщения (HTML-теги `<b>`, `<i>`, `<a>` поддерживаются) |
| `-File <путь>` | Альтернатива `-Text`: прочитать текст из `.md`/`.txt`-файла |
| `-ChatId` | ID канала; по умолчанию `@remixclient_ru` (или `REMIX_TG_CHAT`) |
| `-Silent` | Отправить без звукового уведомления |
| `-DryRun` | Показать, что будет отправлено, без реальной отправки |

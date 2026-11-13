<#
.SYNOPSIS
  Публикация новостей RemixClient в Telegram-канал через Bot API.

.DESCRIPTION
  Отправляет HTML-сообщение в Telegram-канал. Токен берётся из переменной
  окружения REMIX_TG_TOKEN или из файла %USERPROFILE%\.remix\tg-token.txt.
  Chat id берётся из REMIX_TG_CHAT, второго аргумента или по умолчанию
  "@remixclient_ru".

.EXAMPLE
  .\telegram-post.ps1 -Text "Вышла <b>версия 0.2</b>!"
  .\telegram-post.ps1 -File .\changelog.md -Silent
  .\telegram-post.ps1 -Text "Тест" -DryRun
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [string]$Text,

    [Parameter(Mandatory = $false)]
    [string]$File,

    [Parameter(Mandatory = $false)]
    [string]$ChatId,

    [switch]$Silent,
    [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ── Валидация: нужен -Text или -File ──────────────────────────────────────────
if (-not $Text -and -not $File) {
    Write-Error "Укажите -Text `"...`" или -File <путь к .md/.txt>."
    exit 1
}

if ($Text -and $File) {
    Write-Error "Укажите что-то одно: -Text или -File, не оба."
    exit 1
}

# ── Чтение текста из файла ────────────────────────────────────────────────────
if ($File) {
    if (-not (Test-Path $File)) {
        Write-Error "Файл не найден: $File"
        exit 1
    }
    $Text = Get-Content -Path $File -Raw -Encoding UTF8
    if ([string]::IsNullOrWhiteSpace($Text)) {
        Write-Error "Файл пуст: $File"
        exit 1
    }
}

# ── Токен ─────────────────────────────────────────────────────────────────────
$Token = $env:REMIX_TG_TOKEN

if ([string]::IsNullOrWhiteSpace($Token)) {
    $tokenFile = Join-Path $env:USERPROFILE '.remix\tg-token.txt'
    if (Test-Path $tokenFile) {
        $Token = (Get-Content -Path $tokenFile -Raw -Encoding UTF8).Trim()
    }
}

if ([string]::IsNullOrWhiteSpace($Token)) {
    Write-Error @"
Токен Telegram-бота не найден.
Задайте переменную окружения REMIX_TG_TOKEN или положите токен (одну строку)
в файл: $env:USERPROFILE\.remix\tg-token.txt

Подробнее: tools/telegram-post.md
"@
    exit 1
}

# ── Chat id ───────────────────────────────────────────────────────────────────
if ([string]::IsNullOrWhiteSpace($ChatId)) {
    $ChatId = $env:REMIX_TG_CHAT
}
if ([string]::IsNullOrWhiteSpace($ChatId)) {
    $ChatId = '@remixclient_ru'
}

# ── Экранирование HTML ────────────────────────────────────────────────────────
# Telegram HTML-режим требует экранировать &, < и > вне тегов.
# Стратегия: заменяем все &, <, > на сущности, потом восстанавливаем
# допустимые теги (<b>, <i>, <u>, <s>, <a href>, <code>, <pre>).
function Escape-TelegramHtml {
    param([string]$Raw)

    $s = $Raw
    $s = $s.Replace('&', '&amp;')
    $s = $s.Replace('<', '&lt;')
    $s = $s.Replace('>', '&gt;')

    # Восстановить разрешённые теги Telegram
    $allowed = @('b', 'i', 'u', 's', 'code', 'pre', 'em', 'strong', 'ins', 'del', 'tg-spoiler', 'blockquote')
    foreach ($tag in $allowed) {
        $s = $s -replace "(?i)&lt;($tag)&gt;",   '<$1>'
        $s = $s -replace "(?i)&lt;/($tag)&gt;",  '</$1>'
    }
    # <a href="...">
    $s = $s -replace '(?i)&lt;a\s+href=&quot;(.+?)&quot;&gt;', '<a href="$1">'
    $s = $s -replace '(?i)&lt;a\s+href="(.+?)"&gt;',          '<a href="$1">'
    $s = $s -replace '(?i)&lt;/a&gt;',                         '</a>'

    return $s
}

$escapedText = Escape-TelegramHtml $Text

# ── DryRun ────────────────────────────────────────────────────────────────────
if ($DryRun) {
    Write-Host '=== DRY RUN ===' -ForegroundColor Yellow
    Write-Host "Chat:   $ChatId"
    Write-Host "Silent: $($Silent.IsPresent)"
    Write-Host "Token:  $($Token.Substring(0, [Math]::Min(8, $Token.Length)))..."
    Write-Host '--- Текст (HTML) ---'
    Write-Host $escapedText
    Write-Host '--- Конец ---'
    exit 0
}

# ── Отправка ──────────────────────────────────────────────────────────────────
$uri = "https://api.telegram.org/bot$Token/sendMessage"

$body = @{
    chat_id                  = $ChatId
    text                     = $escapedText
    parse_mode               = 'HTML'
    disable_web_page_preview = $true
}

if ($Silent) {
    $body['disable_notification'] = $true
}

$maxRetries = 3

for ($attempt = 1; $attempt -le $maxRetries; $attempt++) {
    try {
        $response = Invoke-RestMethod -Uri $uri -Method Post -Body $body -ContentType 'application/x-www-form-urlencoded; charset=utf-8'

        if ($response.ok -eq $true) {
            $msgId = $response.result.message_id
            Write-Host "✅ Отправлено! message_id = $msgId" -ForegroundColor Green
            exit 0
        }
        else {
            Write-Error "Telegram вернул ok=false: $($response | ConvertTo-Json -Depth 5)"
            exit 1
        }
    }
    catch {
        $statusCode = $null
        $retryAfter = $null
        $errBody    = $null

        if ($_.Exception.Response) {
            $statusCode = [int]$_.Exception.Response.StatusCode

            try {
                $reader  = [System.IO.StreamReader]::new($_.Exception.Response.GetResponseStream())
                $errBody = $reader.ReadToEnd()
                $reader.Close()
            } catch {}
        }

        # Парсим retry_after из JSON-тела ответа
        if ($errBody) {
            try {
                $parsed = $errBody | ConvertFrom-Json
                if ($parsed.parameters.retry_after) {
                    $retryAfter = [int]$parsed.parameters.retry_after
                }
            } catch {}
        }

        switch ($statusCode) {
            401 {
                Write-Error "❌ 401 Unauthorized — токен бота неверный. Проверьте REMIX_TG_TOKEN."
                exit 1
            }
            403 {
                Write-Error "❌ 403 Forbidden — бот не является админом канала $ChatId или канал не существует."
                exit 1
            }
            429 {
                if ($retryAfter -and $attempt -lt $maxRetries) {
                    Write-Warning "⏳ 429 Too Many Requests. Повтор через $retryAfter сек (попытка $attempt/$maxRetries)..."
                    Start-Sleep -Seconds $retryAfter
                    continue
                }
                Write-Error "❌ 429 Too Many Requests. Попробуйте позже (retry_after=$retryAfter)."
                exit 1
            }
            default {
                Write-Error "❌ Ошибка Telegram API (HTTP $statusCode): $errBody`n$($_.Exception.Message)"
                exit 1
            }
        }
    }
}

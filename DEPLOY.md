# Настройка авторизации через Яндекс

## 1. Создание OAuth приложения

1. Перейдите на https://oauth.yandex.ru/client/new
2. Заполните форму:
   - **Название**: Путевые Листы
   - **Для какого сервиса**: Веб-сервисы
   - **Callback URL**: `https://docs.salskayastep.ru/auth/yandex/callback`
3. Нажмите "Создать приложение"
4. Скопируйте **Client ID** и **Client Secret**

## 2. Настройка окружения

```bash
cp .env.example .env
```

Отредактируйте `.env`:
```
YANDEX_CLIENT_ID=ваш_client_id
YANDEX_CLIENT_SECRET=ваш_client_secret
BASE_URL=https://docs.salskayastep.ru
PORT=3000
```

**Важно**: файл `.env` уже в `.gitignore`, он не попадёт в Git!

## 3. Запуск

### Через Docker Compose (рекомендуется):
```bash
docker-compose up -d
```

### Локально:
```bash
cargo run --release
```

## 4. Безопасность

- Каждый пользователь видит только свои данные
- Данные привязаны к yandex_id пользователя
- Сессии хранятся в cookies (httpOnly, secure)
- Секреты не попадают в Git (через .env)

## 5. Обратный прокси (Nginx)

Пример конфигурации для docs.salskayastep.ru:

```nginx
server {
    listen 443 ssl http2;
    server_name docs.salskayastep.ru;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}
```

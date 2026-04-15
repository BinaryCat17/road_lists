# Путевые Листы

Приложение для создания и печати путевых листов с авторизацией через Яндекс.

## Возможности

- Авторизация через Яндекс OAuth
- Изолированные данные для каждого пользователя
- Управление водителями и транспортом
- Генерация PDF путевых листов через Typst
- Настройка реквизитов организации
- Значения по умолчанию для быстрого заполнения

## Требования

- Rust 1.75+
- SQLite

## Настройка

1. Скопируйте `.env.example` в `.env`:
   ```bash
   cp .env.example .env
   ```

2. Получите OAuth credentials от Яндекса:
   - Перейдите на https://oauth.yandex.ru/
   - Создайте новое приложение
   - Выберите "Веб-сервисы"
   - Укажите Callback URL: `https://docs.salskayastep.ru/auth/yandex/callback`
   - Сохраните Client ID и Client Secret

3. Заполните `.env`:
   ```
   YANDEX_CLIENT_ID=your_client_id
   YANDEX_CLIENT_SECRET=your_client_secret
   BASE_URL=https://docs.salskayastep.ru
   PORT=3000
   ```

4. Сборка:
   ```bash
   cargo build --release
   ```

5. Запуск:
   ```bash
   ./target/release/road_lists
   ```

## Деплой

### Docker

```bash
docker build -t road_lists .
docker run -d -p 3000:3000 --env-file .env road_lists
```

### Nginx (reverse proxy)

```nginx
server {
    listen 80;
    server_name docs.salskayastep.ru;
    
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

## Безопасность

- `.env` файл содержит секреты и **не должен** попадать в git
- Сессии хранятся в cookies с httpOnly флагом
- Каждый пользователь видит только свои данные
- SQL-инъекции защищены через параметризованные запросы (sqlx)

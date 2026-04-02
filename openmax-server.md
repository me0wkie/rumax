<b>Текущая поддержка openmax не предназначена для прода, только для тестов.</b>

Генерация сертификата:

```sh
openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout key.pem -out cert.pem -days 365 \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost" \
  -addext "basicConstraints=CA:FALSE" \
  -addext "keyUsage=digitalSignature,keyEncipherment" \
  -addext "extendedKeyUsage=serverAuth"
```

<b>В openmax-server:</b>

1. Добавить свой telegramId в .env
2. Написать боту /register
3. Скопировать номер и авторизоваться в клиенте

# Prototipo RabbitMQ 

## Installation

### Frontend

```bash
wasm-pack build
cd website/
npm install
npm run build
# The website files are now available in dist/*
# Move the static files
mkdir /home/user/www
mv dist/* /home/user/www
```
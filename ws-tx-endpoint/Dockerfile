FROM rust:1.75 as builder

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

WORKDIR /app
COPY Cargo.toml ./
COPY src/ ./src/

# Build WASM package
RUN wasm-pack build --target web --out-dir pkg

# Web server stage
FROM nginx:alpine

COPY --from=builder /app/pkg /usr/share/nginx/html/pkg
COPY index.html /usr/share/nginx/html/
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 8000

CMD ["nginx", "-g", "daemon off;"]

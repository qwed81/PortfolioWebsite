FROM denoland/deno:alpine-1.40.2
EXPOSE 8080
WORKDIR /app
USER deno
COPY . .
RUN deno cache main.ts
CMD ["run", "-A", "main.ts"]

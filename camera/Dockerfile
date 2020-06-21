
FROM python:3.8-alpine
WORKDIR /app
COPY camera/main.py .
COPY --from=builder /foo /foo
CMD ["python", "main.py"]

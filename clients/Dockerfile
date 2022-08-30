FROM python:3.10.5-alpine3.15

WORKDIR /app
COPY client.py /app/client.py
RUN chmod +x /app/client.py

ENTRYPOINT ["/app/client.py"]

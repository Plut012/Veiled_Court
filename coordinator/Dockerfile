FROM python:3.12-slim

WORKDIR /app

COPY coordinator/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY coordinator/ ./coordinator/
COPY frontend/ ./frontend/

EXPOSE 8000

CMD ["uvicorn", "coordinator.main:app", "--host", "0.0.0.0", "--port", "8000"]

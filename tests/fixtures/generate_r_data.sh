docker build -t myclim-test ./docker
docker run -v ./data/:/data/ myclim-test:latest
## Docker Instructions

If you have [Docker](https://www.docker.com/) installed, you can run this
in your terminal, when the Dockerfile is inside the `.devcontainer` directory:

```bash
cd .devcontainer/ubuntu
docker compose run -it --rm --build ubuntu
```

This command will put you in a `bash` session in a Ubuntu 24.04 Docker container,
with all of the tools listed in the [Dependencies](#dependencies) section already installed.

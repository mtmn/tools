```bash
# start containers (both, mysql, elastic)
podman-compose --profile both up -d
# teardown
podman-compose --profile both down -v 
```

## replace mysql with postgres

```bash
ddev delete -Oy
ddev config --database=postgres:16
ddev start

ddev craft install
```

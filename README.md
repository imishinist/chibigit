# chibigit

It outputs git ls-files --stage.

```bash
diff <(git ls-files --stage) <(cargo run -- 2>/dev/null)
```

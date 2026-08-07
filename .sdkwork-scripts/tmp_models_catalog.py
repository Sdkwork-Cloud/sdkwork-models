import io, re

def drop_fns(src, names, prefix='pub fn '):
    """Drop each top-level `<prefix><name>(` ... balanced close (plus preceding blank line)."""
    for name in names:
        marker = prefix + name + '('
        idx = src.find(marker)
        if idx == -1:
            continue
        start = idx
        depth = 0
        j = idx
        while j < len(src):
            ch = src[j]
            if ch == '{':
                depth += 1
            elif ch == '}':
                depth -= 1
                if depth == 0:
                    break
            j += 1
        end = j + 1
        if src[start-2:start] == '\n\n':
            start -= 1
        src = src[:start] + src[end:]
    return src

# ---------- admin_models_list.rs ----------
f = "crates/sdkwork-models-catalog-repository-sqlx/src/admin_models_list.rs"
with io.open(f, "r", encoding="utf-8") as fh:
    c = fh.read()
c = drop_fns(c, ["sqlite_capability_in_clause", "sqlite_vendor_codes_in_clause",
                 "sqlite_modalities_clause", "sqlite_release_stages_clause"])
with io.open(f, "w", encoding="utf-8") as fh:
    fh.write(c)
print("admin_models_list.rs done:", c.count('sqlite'))

# ---------- routing_config_change.rs ----------
f = "crates/sdkwork-models-catalog-repository-sqlx/src/routing_config_change.rs"
with io.open(f, "r", encoding="utf-8") as fh:
    c = fh.read()
c = drop_fns(c, ["record_sqlite_ai_routing_config_change",
                 "bump_sqlite_ai_routing_config_version",
                 "insert_sqlite_ai_routing_config_change_event"], prefix='')
c = c.replace("use sqlx::{Postgres, Sqlite, Transaction};", "use sqlx::{Postgres, Transaction};")
with io.open(f, "w", encoding="utf-8") as fh:
    fh.write(c)
print("routing_config_change.rs done:", c.count('sqlite'))

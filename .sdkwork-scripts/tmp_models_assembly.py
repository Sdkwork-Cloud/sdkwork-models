import io, re

GUARD = ('DatabasePool::Sqlite(_, _) => unreachable!(\n'
         '            "models server assembly requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"\n'
         '        ),')

def drop_block(src, marker, limit=None):
    """Remove the arm block starting at marker line through its balanced close."""
    idx = src.find(marker)
    if idx == -1:
        return src
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
    return src[:idx] + src[j+1:]

def drop_import_sqlite(src):
    # remove Sqlite* names from the use statement lines
    src = re.sub(r'    Sqlite[A-Za-z]+,?\n', '', src)
    src = re.sub(r'    Sqlite[A-Za-z]+,\n', '', src)
    src = re.sub(r', Sqlite[A-Za-z]+', '', src)
    return src

# ---------- bootstrap.rs ----------
f = "crates/sdkwork-api-models-assembly/src/bootstrap.rs"
with io.open(f, "r", encoding="utf-8") as fh:
    c = fh.read()
# 1. import: remove Sqlite names
c = re.sub(r'    PostgresModelRankingsReadStore, SqliteAdminAiResourceStore, SqliteModelCatalogAdminStore,\n    SqliteModelRankingRefreshStore, SqliteModelRankingsReadStore,\n',
           '    PostgresModelRankingsReadStore,\n', c)
# 2. Sqlite arms -> unreachable (two of them, block form)
n1 = c.count('DatabasePool::Sqlite(pool, _) => {')
assert n1 == 2, n1
c = c.replace('DatabasePool::Sqlite(pool, _) => {', 'DatabasePool::Sqlite(_, _) => unreachable!(')
# now the Sqlite block bodies are dead; find the unreachable( ... ) balance and drop the trailing block
# Strategy: replace the whole Sqlite arm block (from 'DatabasePool::Sqlite(_, _) => unreachable!(' to its balanced paren/brace)
out = []
i = 0
while True:
    idx = c.find('DatabasePool::Sqlite(_, _) => unreachable!(', i)
    if idx == -1:
        out.append(c[i:])
        break
    out.append(c[i:idx])
    # find closing of unreachable!( ... )  -> then drop the original arm body up to the closing '}' of the arm block
    j = idx + len('DatabasePool::Sqlite(_, _) => unreachable!(')
    depth = 1
    while j < len(c) and depth > 0:
        if c[j] == '(':
            depth += 1
        elif c[j] == ')':
            depth -= 1
        j += 1
    # after ')' we have ',\n        }' (arm close) - find the arm block '}' that closes the old block
    k = j
    while k < len(c) and c[k] != '}':
        k += 1
    out.append(c[idx:j] + ')')
    i = k + 1
c = ''.join(out)
with io.open(f, "w", encoding="utf-8") as fh:
    fh.write(c)
print("bootstrap.rs done, Sqlite refs:", c.count('Sqlite'))

# ---------- contribution.rs ----------
f = "crates/sdkwork-api-models-assembly/src/contribution.rs"
with io.open(f, "r", encoding="utf-8") as fh:
    c = fh.read()
c = re.sub(r'    PostgresModelRankingsReadStore, SqliteAdminAiResourceStore, SqliteModelCatalogAdminStore, SqliteModelRankingsReadStore,\n',
           '    PostgresModelRankingsReadStore,\n', c)
n1 = c.count('DatabasePool::Sqlite(pool, _) => {')
assert n1 == 1, n1
c = c.replace('DatabasePool::Sqlite(pool, _) => {', 'DatabasePool::Sqlite(_, _) => unreachable!(')
out = []
i = 0
while True:
    idx = c.find('DatabasePool::Sqlite(_, _) => unreachable!(', i)
    if idx == -1:
        out.append(c[i:])
        break
    out.append(c[i:idx])
    j = idx + len('DatabasePool::Sqlite(_, _) => unreachable!(')
    depth = 1
    while j < len(c) and depth > 0:
        if c[j] == '(':
            depth += 1
        elif c[j] == ')':
            depth -= 1
        j += 1
    k = j
    while k < len(c) and c[k] != '}':
        k += 1
    out.append(c[idx:j] + ')')
    i = k + 1
c = ''.join(out)
with io.open(f, "w", encoding="utf-8") as fh:
    fh.write(c)
print("contribution.rs done, Sqlite refs:", c.count('Sqlite'))

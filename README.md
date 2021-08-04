# AtCoder Create Contest
AtCoder のコンテストに Rust で参加する際のディレクトリを簡単に作成することができます
`cargo test` でサンプルケースのテストもできます
提出の際は `src/x.rs` を提出してください
## usage
### directory 作成
```
usage:
  create-contest [{-u|--url} <URL>] [{-n|--name} <NAME>] [{-t|--type} <TYPE>]

args:
  -u --url <URL>   コンテストの URL
  -n --name <NAME> コンテストの名前 (ディレクトリの名前になります)
  -t --type {abc|arc|agc|h-abc|s-abc} コンテストの種類 (h-abc: 平成ABC(6問), s-abc: 昭和ABC(4問))

  --url もしくは --name, --type は必須 ただしスポンサードコンテストでは --type も必須 (zoon-2020 等 URL に abc.. などが含まれないもの)
```
### login
進行中のコンテストのサンプルケース取得に必要です(cookie は保存しますが、password は保存しません)
```
usage:
  create-contest login [{-u|--user} <USER_NAME>]
args:
  -u --user <USER_NAME> AtCoder のユーザーネーム
```

### テストケース作成 (未実装)
`create-contest` する際に `url` を指定していれば自動で生成されますが、`name` と `type` を指定して作成した場合はこちらを使ってください
```
usage:
  create-contest add_test {-u|--url} <URL>

args:
  -u --user <URL> コンテストの URL
```

### test
誤差ジャッジやインタラクティブ・解が複数あるもの には対応していません
```
usage:
  cargo test {_a|_b|...}

args:
  _x テストしたい問題に対して _x とすることでその問題のテストができます 指定しない場合はすべての問題のテストが行われます
```

## example
```
create-contest login -u SSlime
```
```
create-contest login
```

```
create-contest --url https://atcoder.jp/contests/abc212 --type ABC --name abc-212
```
```
create-contest -u https://atcoder.jp/contests/abc212
```
```
create-contest -u https://atcoder.jp/contests/zone2021 --type H-ABC
```

```
create-contest -n abc-212 -t abc
create-contest -u https://atcoder.jp/contests/abc212
```

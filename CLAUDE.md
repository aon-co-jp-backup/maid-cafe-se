# 開発方針 (maid-cafe-se)

共通の開発環境ルール・運用方針は[`open-raid-z/CLAUDE.md`](https://github.com/aon-co-jp/open-raid-z)を正本とし、
エコシステム索引は[`runo`](https://github.com/aon-co-jp/runo)を参照。作業場所は`F:\maid-cafe-se`。

- Androidアプリを最優先。設計は`README.md`、再開点は`PORTING.md`。
- 音源はCC0/パブリックドメインのみ。出典とライセンスを必ず記録する(著作権フリー要件)。
- Googleカレンダーは読み取り専用スコープ。秘密情報はリポジトリに入れない。
- 実機(Android)で動作確認するまで「完成」と報告しない。

## maid-cafe.tokyo 設計作業 / maid-cafe.tokyo design work

- 設計書 `docs/maid-cafe-tokyo-design.md` が正本。RPoem masterのCIがgreenになり、ユーザーが保守終了を明示するまで実装に着手しない。リポジトリ新設・名前変更・デプロイ・pushはユーザー確認後。再開メッセージは `PORTING.md` 末尾。
- The design doc is canonical. No implementation until RPoem master CI is green and the user declares maintenance done. Repo creation/rename/deploy/push only after user confirmation. The resume message is at the end of `PORTING.md`.

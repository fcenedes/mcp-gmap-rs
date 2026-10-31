# פיתוח וגרסאות

סביבת הריצה היא פרויקט Rust עצמאי. CLI, MCP stdio ו-Streamable HTTP משתפים מימושי כלים. אתר התיעוד הוא כלי עריכה ובנייה נפרד; משתמשי הקצה זקוקים רק לקובץ ההפעלה המקומפל.

## בנייה ובדיקה {#build-and-check}

שכפלו את [המאגר](https://github.com/fcenedes/mcp-gmap-rs) והשתמשו בקובץ הנעילה שלו:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
cargo test --doc --all-features --locked
cargo build --release --locked
```

גרסת המינימום המוצהרת היא Rust 1.88; קובצי ההפעלה להפצה נבנים עם Rust 1.94.1. בדיקות ברירת המחדל משתמשות בנתוני דמה מקומיים של Google ואסימונים, ללא פרטי גישה או רשת ל-Google. CI בודק גם Linux/macOS, גרסת Rust המינימלית, rustdoc יציב ולילי, חבילת מקור, בדיקות Docker/MCP/TLS וקובצי Kubernetes.

לאחר בדיקה מקומית, `cargo clean` מפנה פלט בנייה שנוצר; בניות הבאות יקמפלו אותו מחדש.

## שמירה על מפרט מלא {#keep-the-reference-complete}

מדריך הכלים הקנוני משלב סכמות אמיתיות מקובץ ההפעלה עם ברירות מחדל, תוצאות והערות התנהגות שנבדקו:

```sh
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs --check --verify-examples
```

הבודק מסיר פרטי גישה, מבטל טעינת dotenv ומאמת את כל הדוגמאות בגבול היעדר המפתח בלי לפנות ל-Google. עדכנו את הערות ה-JSON שנבדקו כשפרמטרים או התנהגות משתנים.

בדיקות חיות הן אופציונליות ועשויות להיות כרוכות בתשלום. עם מפתח בדיקה שהוגדר במפורש, הריצו `cargo test --locked --test live live_maps_ -- --ignored --test-threads=1`. שש בדיקות OAuth משתמשות במסנן `live_oauth_` ומסירות את המפתח מתהליכי המשנה. אין לשמור פרטי גישה במאגר או להפעיל בדיקות חיות ב-CI רגיל.

## פרסום גרסה {#publish-a-version}

עדכנו יחד את גרסת Cargo, קובץ הנעילה, יומן השינויים, דוגמאות ההתקנה והערות הגרסה. דחפו והמתינו ל-CI, ואז צרו תג מתועד תואם `vX.Y.Z`. תהליך Release בודק את התג המדויק, בונה שלושה יעדים טבעיים, מאמת ארכיונים וסכומי ביקורת ומפרסם את הטיוטה המוכנה.

תהליך **Publish crate** הנפרד דורש את גרסת GitHub שכבר פורסמה. הפעילו אותו עבור אותו תג. שמרו `CARGO_REGISTRY_TOKEN` בעל היקף מוגבל בסביבת GitHub בשם `crates-io`, ולא במאגר. docs.rs נבנה אוטומטית אחרי פרסום ל-crates.io; גרסת GitHub בלבד אינה מעדכנת תיעוד Rust מתארח. בדקו את הבנייה המתארחת בפועל לאחר העלאה.

ראו [נוהלי שחרור](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/releases.md), [רישום הבדיקות](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/validation.md) ו[נקודת הכניסה הציבורית ל-Rust](https://docs.rs/mcp-gmap-rs/latest/mcp_gmap_rs/fn.run.html).

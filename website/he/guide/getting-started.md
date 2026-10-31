# תחילת העבודה

התקינו קובץ הפעלה אחד, הגדירו גישה ל-Google והשתמשו באותם 23 כלים מהמסוף או מלקוח MCP. מדריך זה מתאר את v0.3.1.

## התקנה {#install}

קובצי הפעלה מוכנים ל-Linux x86_64, macOS Apple Silicon ו-macOS Intel זמינים ב-[GitHub Releases](https://github.com/fcenedes/mcp-gmap-rs/releases). הורידו את הארכיון המתאים, אמתו מול `SHA256SUMS`, חלצו ומקמו את `mcp-gmap-rs` ב-`PATH`. קובצי Linux דורשים glibc 2.35 ומעלה. קובצי macOS אינם חתומים או מאושרי notarization.

עם Rust 1.88 ומעלה, התקינו מ-crates.io:

```sh
cargo install mcp-gmap-rs --version 0.3.1 --locked
mcp-gmap-rs --version
```

לא נדרשת סביבת ריצה של Rust, Python או JavaScript אחרי בניית קובץ ההפעלה. המארח זקוק לתעודות CA, ל-DNS ולגישת HTTPS יוצאת ל-Google.

## הגדרת גישה {#configure-access}

[צרו מפתח מוגבל והפעילו שירותים](./google-cloud), או הגדירו [OAuth לחשבון שירות](./authentication). שמרו פרטי גישה בקובץ `.env` המוחרג מ-Git בתיקיית העבודה:

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

משתני הסביבה גוברים על הקובץ. שמרו פרטי גישה מחוץ לבקרת המקור ולשכבות התמונה. תיקיית העבודה חשובה: אין חיפוש אחר `.env` בתיקיות אב.

## גילוי, בדיקה וקריאה {#discover-check-call}

גילוי ואבחון לא מקוון אינם שולחים בקשות ל-Google:

```sh
mcp-gmap-rs tools
mcp-gmap-rs schema search-places
mcp-gmap-rs doctor
```

לאחר מכן בצעו קריאה ראשונה העשויה להיות כרוכה בתשלום:

```sh
mcp-gmap-rs search-places --query 'coffee near Gare de Lyon, Paris'
```

התוצאה היא JSON. שמרו מזהי מקומות שהוחזרו לפרטים בהמשך במקום לפתור שוב ושוב את אותם שמות. קראו [שימוש ב-CLI](./cli) לאוטומציה או [שילוב MCP](./mcp) לחיבור סוכן. [המתכונים](./recipes) מראים איך לשלב כלים; [המפרט המלא](../reference/tools) מתעד כל פרמטר, ברירת מחדל ותוצאה.

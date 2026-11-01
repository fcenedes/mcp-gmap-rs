# שילוב MCP

חברו מארח MCP דרך stdio לתהליך משנה מקומי או דרך Streamable HTTP לשירות ארוך טווח. שניהם מציגים אותו קטלוג של כלים מופעלים וסכמות.

## stdio מקומי {#local-stdio}

תצורת לקוח MCP כללית:

```json
{
  "mcpServers": {
    "google-maps": {
      "command": "/absolute/path/to/mcp-gmap-rs",
      "args": ["--env-file", "/secure/mcp-gmap/runtime.env", "serve", "stdio"]
    }
  }
}
```

השתמשו בפורמט ההגדרות של המארח ובנתיב מוחלט לקובץ ההפעלה. קובץ dotenv הנבחר חייב להתקיים. לחלופין אפשר להזריק משתני סביבה מהמארח. זו תצורת שרת MCP עצמאי; אין צורך בתוסף ריצה ייעודי לעוזר מסוים.

```sh
mcp-gmap-rs serve stdio
```

stdin ו-stdout נושאים רק הודעות MCP JSON-RPC מופרדות בשורות. אבחון נכתב ל-stderr. הפעלת הקובץ ללא ארגומנטים מפעילה אותה תעבורה. מארח stdio בקונטיינר חייב לחבר stdin בלי להקצות TTY.

## Streamable HTTP {#streamable-http}

```sh
mcp-gmap-rs serve http --listen 127.0.0.1:3000
```

הגדירו את כתובת Streamable HTTP של לקוח MCP ל-**`http://127.0.0.1:3000/mcp`**. אם המארח תומך בהגדרות MCP מרוחקות, הגדירו נקודת קצה וכותרת הרשאה:

```json
{
  "url": "https://maps.example.com/mcp",
  "headers": {
    "Authorization": "Bearer your-mcp-http-token"
  }
}
```

העטיפה המדויקת של הגדרה זו תלויה במארח MCP. השרת תומך במשא ומתן על הפרוטוקול ובפעולות הסשן המתאימות POST, GET/SSE ו-DELETE דרך Rust MCP SDK. השתמשו בלקוח MCP במקום להתייחס לבקשת JSON-RPC POST בודדת כסשן שלם.

## כתובות האזנה וגישה {#bind-addresses-and-access}

מאזינים שאינם loopback דורשים `MCP_HTTP_TOKEN`. הלקוחות שולחים אסימון זה כ-bearer, בנפרד מ-Google OAuth או ממפתחות API. הוסיפו במפורש את ערך Host של נקודת הקצה ואת מקורות הדפדפן בפועל:

```sh
mcp-gmap-rs serve http --listen 0.0.0.0:3000 \
  --allowed-host maps.example.com \
  --allowed-origin https://app.example.com:443
```

ברירת המחדל דוחה כותרות Origin קיימות אם אינן ברשימת ההיתר. גופי בקשות מוגבלים ל-12 MiB. `/health` בודק חיות תהליך בלי לפנות ל-Google. לגישה ציבורית הגדירו [TLS/mTLS מובנה או שער](./production). שרתי proxy חייבים לשמר הרשאות וכותרות פרוטוקול וסשן MCP ולאפשר הזרמה ללא אגירה. סשנים וזהות Google מקומיים לתהליך; ראו [פריסה](./deployment) לפני הוספת עותקים.

# תפעול בייצור

מדריך זה מכסה את v0.3.x. כל כלי Google משתפים אותו מימוש ב-CLI, ב-MCP stdio וב-Streamable HTTP.

## TLS מובנה ו-TLS הדדי {#native-tls-and-mutual-tls}

מאזין HTTP יכול להגיש HTTPS ישירות באמצעות rustls ו-TLS 1.2/1.3:

```sh
mcp-gmap-rs serve http --listen 0.0.0.0:3443 \
  --tls-cert /etc/mcp-gmap/tls/tls.crt \
  --tls-key /etc/mcp-gmap/tls/tls.key \
  --allowed-host maps.example.com:3443
```

ספקו `MCP_HTTP_TOKEN` בנפרד דרך הסביבה או Secret. לקוחות מתחברים ל-`https://maps.example.com:3443/mcp`, מאמתים את תעודת השרת ושולחים `Authorization: Bearer <token>`. קובצי התעודות כוללים תעודת קצה PEM ואחריה שרשרת ביניים; המפתח הפרטי חייב להתאים. קובץ ההפעלה אינו מנפיק תעודות ואינו מממש ACME. השתמשו ב-PKI שלכם, ב-cert-manager או בשער TLS להנפקה ולחידוש.

הוסיפו `--tls-client-ca /etc/mcp-gmap/tls/client-ca.pem` לדרישת תעודת לקוח חתומה בידי CA זה. mTLS מובנה חל בכל הנתיבים, כולל `/health` ו-`/metrics`, ואינו מחליף אסימון HTTP. כל זהות לקוח תקינה תחת שורשי האמון המוגדרים מתקבלת בשכבת TLS; אין הרשאות כלים לפי תעודה או בדיקות רשימות ביטול.

ב-Linux/macOS החליפו את הקבצים באופן אטומי ושלחו `SIGHUP` כדי לטעון יחד את התעודה, המפתח ו-CA הלקוח האופציונלי. חיבורים קיימים שומרים את מצב TLS שלהם; לחיצות יד חדשות משתמשות בהחלפה. אם הטעינה נכשלת, התצורה הקודמת נשארת פעילה ונרשמת שגיאה ללא חומר מפתחות. אפשר לטעון כך מחדש קובצי Kubernetes Secret מותקנים אחרי עדכון ההקרנה שלהם. שאר ההגדרות עדיין דורשות הפעלה מחדש.

עבור UID/GID 10001 בקונטיינר, התקינו מפתחות לקריאה בלבד עם הרשאות המאפשרות לזהות לקרוא אותם; השתמשו בקבוצה מתאימה וב-`fsGroup` עבור Secrets מוקרנים. עם TLS מובנה, בדיקות HTTP של Kubernetes חייבות להשתמש ב-`scheme: HTTPS`. בדיקות HTTP של kubelet אינן יכולות לספק תעודת לקוח mTLS; השתמשו בבדיקת TCP בפריסה זו, בידיעה שהיא בודקת את המאזין ולא את TLS או Google.

## שער TLS ב-Kubernetes {#kubernetes-tls-gateway}

[קובץ השער](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/gateway.yaml) מספק שער HTTPS ו-HTTPRoute ל-Deployment שב[קובץ Kubernetes הבסיסי](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/kubernetes.yaml). התקינו תחילה CRDs של Gateway API ובקר תואם. החליפו `your-gateway-class` ו-`maps.example.com`, וצרו Secret מסוג `kubernetes.io/tls` בשם `mcp-gmap-tls` באותו מרחב שמות. הדוגמה משתמשת בשדות Gateway/HTTPRoute v1 יציבים ובפסקי זמן של HTTPRoute הנתמכים מ-Gateway API v1.2. בדקו את תמיכת הבקר בפסקי זמן.

```sh
kubectl create secret tls mcp-gmap-tls --cert=/secure/fullchain.pem --key=/secure/key.pem
kubectl set env deployment/mcp-gmap-rs \
  MCP_HTTP_ALLOWED_HOSTS='maps.example.com,mcp-gmap-rs:3000,localhost:3000,127.0.0.1:3000'
kubectl apply -f deploy/gateway.yaml
kubectl get gateway,httproute
```

שמרו את הגדרת המארח המותר גם ב-ConfigMap או במקור הפריסה. הוסיפו מקורות דפדפן אמיתיים עם `MCP_HTTP_ALLOWED_ORIGINS` לפי הצורך. הנתיב חושף רק `/mcp`; בריאות ומדדים נשארים בשירות הפרטי. כותרות הרשאה וסשן/פרוטוקול MCP חייבות לעבור ללא שינוי. בטלו אגירה ומטמון של תשובות והגדירו זמני הזרמה וחוסר פעילות בבקר. הנתיב מבטל מועדי בקשה ובקשת backend; הגדירו בשער מגבלות חיבור וחוסר פעילות המתאימות ללקוחות.

תצורה זו מסיימת TLS בשער ומשתמשת ב-HTTP ל-backend של ClusterIP. השתמשו ב-TLS מובנה יחד עם BackendTLSPolicy הנתמך בבקר או ב-mesh אם גם החיבור מהשער ל-Pod חייב להיות מוצפן. הגבילו גישה ישירה ל-Pod ול-Service במדיניות הרשת של האשכול. TLS אינו מספק כניסת MCP OAuth: קובץ ההפעלה משתמש באסימון bearer משותף ובזהות Google אחת לתהליך.

ראו [מדריך TLS לשער](https://gateway-api.sigs.k8s.io/guides/user-guides/tls/) ו[מדריך פסקי זמן של HTTPRoute](https://gateway-api.sigs.k8s.io/guides/http-timeouts/).

## מגבלות, ניסיונות חוזרים ומועדי סיום {#limits-retries-and-deadlines}

| משתנה | ברירת מחדל | אפשרות | משמעות וערכים מותרים |
| --- | --- | --- | --- |
| `GOOGLE_MAPS_MAX_REQUESTS` | `200` | `--max-requests` | תקציב ניסיונות Google HTTP לכל קריאת כלי, מספר שלם 1–10000. משותף לענפים משולבים; ניסיונות חוזרים צורכים מהתקציב. |
| `GOOGLE_MAPS_MAX_RETRIES` | `2` | `--max-retries` | ניסיונות נוספים לכשלים זמניים לכל בקשת Google, מספר שלם 0–5. אפס פירושו ניסיון ראשוני אחד. הניסיונות נשארים בתוך פסק הזמן והתקציב. |
| `GOOGLE_MAPS_TOOL_TIMEOUT` | `120` | `--tool-timeout` | מועד כולל של הכלי בשניות, מספר שלם 1–3600. כולל השגת פרטי גישה, המתנה, קריאות API וניסיונות חוזרים. |
| `MCP_HTTP_RATE_LIMIT` | `120` | HTTP `--rate-limit` | בקשות `/mcp` מאומתות לדקה לכל תהליך, מספר שלם 0–1000000. אפס מבטל. קיבולת ההתפרצות שווה לקצב עם מילוי רציף. |
| `MCP_METRICS_ENABLED` | `false` | HTTP `--metrics [true\|false]` | הפעלת Prometheus ב-`/metrics`. `--metrics` ללא ערך פירושו true. משתמש באותו אסימון bearer אם הוגדר, ו-mTLS מובנה חל גם כאן. |
| `MCP_LOG_FORMAT` | `text` | `--log-format text\|json` | הערכים `text` או `json`. אבחון נכתב תמיד ל-stderr, ומשאיר stdout ל-JSON של CLI או לתעבורת MCP. |
| `MCP_TLS_CERT` | לא מוגדר | HTTP `--tls-cert` | תעודת קצה של שרת ב-PEM עם שרשרת ביניים. יש לספק יחד מפתח פרטי תואם. |
| `MCP_TLS_KEY` | לא מוגדר | HTTP `--tls-key` | נתיב מפתח פרטי של שרת ב-PEM. שמרו קריא לזהות הריצה ומחוץ למקור ולתמונות. |
| `MCP_TLS_CLIENT_CA` | לא מוגדר | HTTP `--tls-client-ca` | חבילת CA ללקוחות ב-PEM המחייבת תעודות לקוח בכל נתיבי המאזין. אינה מחליפה אסימון bearer. |

סדר ההגדרות הוא ברירות מחדל, dotenv אופציונלי, סביבה ואפשרויות. יש לספק יחד תעודת TLS ומפתח. נתיבי TLS ריקים בסביבה מנקים את ערכי הקובץ. `GOOGLE_MAPS_TIMEOUT` מגביל כל בקשת Google HTTP כולל המתנה לסמפור, ניסיונות חוזרים, השהיה וקריאת תשובה; להשגת OAuth יש גבול נפרד באותו ערך. המועד הכולל של הכלי כולל את שניהם.

לכל קריאה תקציב בקשות חדש המשותף לכל הענפים המקבילים שלה. חמש בקשות Google API יכולות לרוץ במקביל בכל תהליך. כשהתקציב נגמר, ניסיונות נוספים נכשלים מקומית. כלים משולבים עשויים להחזיר פלט מוצלח המכיל שגיאות או חריגת תקציב לכל פריט; בדקו שדות אלה. המגבלה היא על מספר הבקשות ולא על כסף: לבקשות ולשדות Google מחירים שונים, ובקשת מטריצה אחת יכולה לחייב מספר רכיבים.

בקשות Google לקריאה בלבד מנסות שוב ב-HTTP 429, 500, 502, 503 ו-504 ובכשלי רשת. ניסיונות חוזרים מכבדים `Retry-After` בשניות או בתאריך HTTP, ואחרת משתמשים בהשהיה מעריכית עם אקראיות. הם נשארים בתוך מועד הבקשה ותקציב הקריאה. שגיאות HTTP אחרות, כולל 400/401/403, אינן מנוסות שוב. כשל אימות אינו מחליף בשקט למצב מפתח API. הגדירו אפס ניסיונות חוזרים לניסיון יחיד. שגיאות JSON פנימיות ישנות של Google מדווחות ללא ניסיון חוזר.

מגביל HTTP הוא דלי אסימונים שקיבולת ההתפרצות שלו שווה לקצב לדקה והוא מתמלא ברציפות. הוא כולל אתחול, POST, GET ו-DELETE ב-`/mcp` בכל הסשנים. בקשות לא מורשות אינן צורכות מהדלי. דחייה מחזירה HTTP 429 עם `Retry-After`. בדיקות בריאות ואיסוף מדדים פטורים. המגביל כללי לתהליך ולא לכל משתמש או IP, ואינו סומך על כותרות IP מועברות. השתמשו באימות בשער ובמגבלות לכל לקוח בפריסות משותפות.

## מדדים ואירועי אבחון {#metrics-and-diagnostic-events}

```sh
mcp-gmap-rs --log-format json serve http --metrics --rate-limit 120
```

`GET /metrics` משתמש באותו אסימון bearer של `/mcp` אם הוגדר. במאזין loopback ללא אימות גם הוא אינו מאומת. הוא כבוי כברירת מחדל, ו-mTLS מובנה חל כשהוא מופעל. חשפו אותו רק לאוספי מדדים מהימנים. נקודת Prometheus הטקסטואלית מייצאת:

- `mcp_gmap_tool_calls_total{tool,outcome}` לקריאות כלים שהושלמו;
- `mcp_gmap_tool_duration_seconds{tool,outcome}` כהיסטוגרמה;
- `mcp_gmap_google_requests_total{api,status}` לניסיונות Google HTTP, כולל ניסיונות חוזרים;
- `mcp_gmap_http_rate_limited_total` לבקשות HTTP שנדחו.

התוויות כוללות שמות כלים מובנים, שמות מארחי API, קודי HTTP ותוצאות קבועות. הן לעולם אינן כוללות כתובות, קואורדינטות, פרטי גישה, אסימונים או מזהי סשן. קריאה משולבת עם שגיאות לפריטים עשויה להיחשב מוצלחת ברמה העליונה; בדקו תוצאה ומדדי מצב השרת במעלה הזרם. מדדים מתאפסים בהפעלה מחדש.

הגדירו `RUST_LOG=info` לצפייה באירועי סיום כלי עם שם, הצלחה וזמן שחלף. ארגומנטים וגופי תשובות Google אינם נכללים. כתובת האזנה בהפעלה נכתבת בנפרד ל-stderr. stdout נשאר ל-JSON של CLI או לתעבורת MCP.

## Doctor {#doctor}

```sh
mcp-gmap-rs doctor
# Explicit opt-in: makes potentially billable requests.
mcp-gmap-rs doctor --live
# Check only the API dependencies of selected tools:
mcp-gmap-rs --enabled-tools maps_pollen,maps_area_insights doctor --live
```

ללא `--live`, doctor בודק תצורת פרטי גישה מקומית, הגדרות HTTP והתאמת קובץ ומפתח TLS בלי בקשות רשת. הוא אינו מוכיח תוקף מפתח, IAM, הפעלת API, אמון/שם מארח/תוקף תעודה או גישת ADC. גילוי ADC מדווח כ-`not_tested` עד בקשה חיה. Doctor אינו מדפיס ערכי פרטי גישה או JSON שלהם. דוח JSON נכתב ל-stdout; בדיקה כושלת מחזירה קוד יציאה 1. נתחו `checks[].status`: הערכים `pass`, `fail` או `not_tested`.

מצב חי מפעיל קריאת כלי מייצגת אחת לכל Google API ייחודי שהכלים המופעלים משתמשים בו, עם הגדרות רגילות של ניסיונות חוזרים, מועדים ותקציב. הוא נמנע מפעולות משולבות יקרות. Routes משתמש בקואורדינטות מדויקות ו-Roads בחיפוש הדרך הקרובה. הבדיקות משתמשות באתרים ציבוריים בפריז, ולא בכתובות שסופקו בידי משתמש. הצלחה פירושה ש-Google קיבלה בדיקה זו, לא שכל פעולה או אזור זמינים.


## כיבוי ושינויי תצורה {#shutdown-and-configuration-changes}

Ctrl-C ו-SIGTERM מכבים את שרת HTTP ואת סשני MCP הפעילים. דוגמת Kubernetes נותנת לתהליך 30 שניות. שאר הגדרות היישום, כולל שינויי Secret מוזרקים בסביבה, דורשות הפעלה מחדש; רק קובצי תעודת TLS מובנית, מפתח ו-CA לקוח תומכים ב-SIGHUP. סשנים ומדדים נשמרים בזיכרון ואובדים בהפעלה מחדש. הפעילו עותק אחד או הגדירו שיוך סשנים; לקוחות חייבים להתחבר מחדש כשהתהליך שבבעלות הסשן מסתיים.

ראו [פריסה](./deployment) להרשאות קונטיינרים, בדיקות והתקנות פרטי גישה בזמן ריצה, ו[מפרט התצורה](../reference/configuration) לכל הגדרה ואפשרות.

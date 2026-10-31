# תצורה

CLI ושתי תעבורות MCP משתמשים באותה תצורת הפעלה. בחרו קובץ לעבודה מקומית, הזרקת משתני סביבה לקונטיינרים ואפשרויות לעקיפות מפורשות ברמת התהליך.

## סדר קדימויות {#precedence}

הערכים מוחלים בסדר הבא, כששכבות מאוחרות גוברות על קודמות:

**ברירות מחדל מובנות → קובץ dotenv אופציונלי → סביבת התהליך → אפשרויות CLI**

קובץ ברירת המחדל הוא `.env` בתיקיית העבודה הנוכחית. אין חיפוש בתיקיות אב. היעדר קובץ ברירת המחדל מותר; קובץ לא קריא או לא תקין מכשיל את ההפעלה בלי להדפיס את תוכנו.

```sh
mcp-gmap-rs --env-file /etc/mcp-gmap/runtime.env serve http
mcp-gmap-rs --no-env-file tools
```

גם `MCP_ENV_FILE` בוחר קובץ, אך `--env-file` גובר עליו. קובץ שנבחר במפורש חייב להתקיים. `--no-env-file` מבטל טעינת קובץ, לרבות `MCP_ENV_FILE`, ומתנגש עם `--env-file`. ניתוח dotenv אינו משנה את סביבת התהליך.

## עקיפת תצורה מותקנת {#override-a-mounted-configuration}

```dotenv
GOOGLE_MAPS_LANGUAGE=en
GOOGLE_MAPS_TIMEOUT=30
GOOGLE_MAPS_ENABLED_TOOLS=maps_search_places,maps_place_details
```

כשהקובץ מותקן, משתנה סביבה משנה את השפה ואפשרות משנה את פסק הזמן:

```sh
GOOGLE_MAPS_LANGUAGE=fr mcp-gmap-rs \
  --env-file /etc/mcp-gmap/runtime.env --timeout 15 tools
```

אפשרויות `--allowed-host` ו-`--allowed-origin` חוזרות מחליפות את כל הרשימה המקבילה בקובץ או בסביבה. גם ערך סביבה ריק במפורש גובר על הקובץ: הוא יכול לנקות אסימון או רשימה אופציונליים, אך פסק זמן ריק אינו תקין.

## שליטה בכלים הזמינים {#control-the-tool-surface}

`GOOGLE_MAPS_ENABLED_TOOLS` או `--enabled-tools` מקבל שמות לוגיים מופרדים בפסיקים כגון `maps_weather,maps_air_quality`, או `*` לכל הכלים. שמות לא מוכרים מכשילים הפעלה. כלים מושבתים אינם מופיעים בגילוי ואי אפשר לקרוא להם. גילוי ובדיקת סכמות אינם דורשים פרטי גישה, אך פעולת API דורשת את פרטי הגישה המתאימים של Google.

הגדרות היישום נקראות בהפעלה. הפעילו מחדש לאחר שינוי משתני סביבה, JSON של פרטי גישה או Secret/ConfigMap של Kubernetes. קובצי תעודת TLS מובנית, המפתח ו-CA ללקוחות הם חריג: הם תומכים ב[טעינה מחדש ב-SIGHUP](./production#native-tls-and-mutual-tls).

ראו [מפרט התצורה המלא](../reference/configuration) לכל 21 ההגדרות, ברירות המחדל, האפשרויות והטווחים. בחירת שפת האתר אינה משנה את `GOOGLE_MAPS_LANGUAGE`, השולט בטקסט התשובות של Google.

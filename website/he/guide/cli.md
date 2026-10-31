# ממשק שורת הפקודה

השתמשו ב-CLI לשאילתות חד-פעמיות, לאוטומציה במעטפת ולהפעלת תהליכי משנה מובנית. כל כלי משתף את סכמת הקלט ואת המימוש עם MCP.

## גילוי הממשק המותקן {#discover-the-installed-interface}

```sh
mcp-gmap-rs --version
mcp-gmap-rs --help
mcp-gmap-rs tools
mcp-gmap-rs schema search-nearby
mcp-gmap-rs search-nearby --help
```

`tools` מציג את הכלים המופעלים ואת הסכמות שלהם. בדקו את הגרסה המותקנת במקום להניח שכל יכולות המדריך החדש זמינות. הפעלה ללא תת-פקודה מפעילה את שרת stdio, ולכן באוטומציה חד-פעמית יש תמיד לציין פקודה.

## אפשרויות או JSON משותף {#flags-or-shared-json}

פקודות הכלים משתמשות באפשרויות kebab-case; אובייקטים ומערכים נמסרים כ-JSON. אפשרויות בוליאניות מקבלות `true` או `false`, ואפשרות בוליאנית ללא ערך פירושה `true`.

```sh
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 500 --open-now
```

ממשק `call` הכללי מקבל שם לוגי של כלי MCP או שם פקודת CLI, ומשתמש באותו אובייקט ארגומנטים של MCP:

```sh
mcp-gmap-rs call maps_geocode --args '{"address":"Paris, France"}'
printf '%s' '{"address":"Paris, France"}' | mcp-gmap-rs call geocode --args -
```

כשמפעילים תהליך משנה מקוד, העבירו JSON מסודר כארגומנט נפרד. הימנעו משילוב טקסט לא מהימן במחרוזות פקודות מעטפת. לפרמטרים ברמה העליונה יש אפשרויות; שדות מקוננים נשארים באובייקטי JSON.

## פלט וטיפול בכשלים {#output-and-failure-handling}

תוצאות רגילות הן JSON ב-stdout. שגיאות נכתבות ל-stderr ומחזירות קוד יציאה שאינו אפס. `doctor` כותב את דוח ה-JSON ל-stdout גם כשהבדיקות נכשלות. תוצאות אצווה, מטריצה ופעולות משולבות עשויות לכלול שגיאות לכל פריט גם אם הפקודה הסתיימה בהצלחה; בדקו אותן לפני שמחשיבים את כל הפעולה למושלמת.

```sh
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --output place.json
mcp-gmap-rs static-map --center Paris --zoom 12 --output paris.png
```

`--output` כותב JSON לכלים רגילים ו-PNG גולמי למפה סטטית. בלעדיו, פלט המפה הסטטית ב-CLI מכיל base64. זמנים משתמשים בדרך כלל ב-RFC 3339 עם אזור זמן מפורש; `timestamp` האופציונלי של Time Zone משתמש ב**מילישניות** Unix. ראו [כל הכלים והדוגמאות](../reference/tools).

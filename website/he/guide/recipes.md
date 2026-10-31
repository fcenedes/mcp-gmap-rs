# מתכונים

שלבו קריאות קטנות ומפורשות סביב מיקום אחד או רשימה מצומצמת של מזהי מקומות שהוחזרו. דוגמאות אלה שולחות בקשות Google העשויות להיות כרוכות בתשלום. בדקו תחילה [אימות זהות](./authentication); כלים משולבים עשויים להשתמש בכמה ממשקים ולהחזיר שגיאות חלקיות.

## מציאת בית קפה והשוואת הנסיעה {#find-a-cafe-and-compare-the-trip}

חפשו ברדיוס ממשי, הרחיבו רק מזהים מהרשימה המצומצמת והשוו נסיעות. `keyword` ב-Nearby הוא סוג מקום של Google ולא טקסט חופשי.

```sh
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 500 --open-now --min-rating 4
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --max-photos 0
mcp-gmap-rs distance-matrix --origins '["48.8584,2.2945"]' \
  --destinations '["48.8606,2.3376","48.8530,2.3499"]' --mode walking
```

השתמשו ב-[`search-places`](../reference/tools#maps_search_places) לכוונה בשפה טבעית, ב-[`compare-places`](../reference/tools#maps_compare_places) להתאמות מפורטות עם מרחק נהיגה אופציונלי, או ב-[`explore-area`](../reference/tools#maps_explore_area) לחקירה לפי קטגוריות. דף חיפוש אינו מלאי ממצה. שמרו קישורי מקור של Google, מחברי ביקורות וגילויי AI.

## תכנון עצירות ומציאת סטייה {#plan-stops-and-find-a-detour}

```sh
mcp-gmap-rs plan-route \
  --stops '["48.8584,2.2945","48.8606,2.3376","48.8530,2.3499"]' \
  --mode walking --optimize false
mcp-gmap-rs search-along-route --text-query cafes \
  --origin '48.8584,2.2945' --destination '48.8606,2.3376' \
  --mode walking --max-results 3
```

[`directions`](../reference/tools#maps_directions) מטפל בנסיעה אחת עם אפשרויות זמן והימנעות; [`plan-route`](../reference/tools#maps_plan_route) שומר על נקודות הקצה באופטימיזציה של עצירות ביניים. תחבורה ציבורית מקבלת רק שתי עצירות בסך הכול. חיפוש לאורך מסלול עוקב אחר קו מרובה נקודות במקום להחליפו במעגל אמצע. ברירת המחדל היא הליכה, ולכן בחרו תמיד את אופן הנסיעה הרצוי.

## פתרון מיקום פעם אחת ובדיקת תנאים מקומיים {#resolve-once-inspect-local-conditions}

```sh
mcp-gmap-rs geocode --address 'Eiffel Tower, Paris, France'
mcp-gmap-rs weather --latitude 48.8584 --longitude 2.2945 \
  --type forecast_daily --forecast-days 3
mcp-gmap-rs air-quality --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs pollen --latitude 48.8584 --longitude 2.2945 --days 3
mcp-gmap-rs timezone --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs elevation --locations '[{"latitude":48.8584,"longitude":2.2945}]'
```

השתמשו שוב בקואורדינטות במקום לחזור על פתרון מיקום. הבדילו בין תצפיות, תחזיות והמלצות ספק; הכיסוי עשוי להשתנות. [`reverse-geocode`](../reference/tools#maps_reverse_geocode) פותר קואורדינטות לכתובת; [`batch-geocode`](../reference/tools#maps_batch_geocode) פותר עד 50 כתובות עם שגיאות לכל כתובת. חותמת הזמן האופציונלית של Time Zone היא במילישניות Unix; הגובה במטרים.

## אימות כתובת למשלוח {#validate-a-postal-address}

```sh
mcp-gmap-rs validate-address --address-lines '["5 Avenue Anatole France"]' \
  --locality Paris --postal-code 75007 --region-code FR
```

בדקו את מסקנת האימות, תיקונים ורכיבים לא פתורים. אימות כתובת שונה מקידוד גאוגרפי ואינו מבטיח משלוח. USPS CASS מופעל במפורש ודורש `regionCode` בערך US או PR במימוש זה.

## ספירת מקומות במקום ספירת תוצאות חיפוש {#count-places-without-counting-search-hits}

```sh
mcp-gmap-rs area-insights \
  --area '{"circle":{"latLng":{"latitude":48.8584,"longitude":2.2945},"radius":500}}' \
  --included-types '["restaurant"]' --min-rating 4
```

ספירות מצטברות משתמשות בדרך כלל בייצוג מחרוזת int64 של Google. מזהי מקומות אופציונליים זמינים רק כשהספירה אינה עולה על 100; פרטים אינם נשלפים אוטומטית. לניתוח נראות שהתבקש במפורש, [`local-rank-tracker`](../reference/tools#maps_local_rank_tracker) מודד דירוגי מילות מפתח על רשת. התחילו במילה אחת וברשת 3×3; דירוג חיפוש אינו ציון ביקורות, ויש להשאיר נקודות כושלות גלויות.

## התאמת נתוני GPS ושמירת מפה {#match-gps-data-and-save-a-map}

השתמשו ב-[`snap-to-roads`](../reference/tools#maps_snap_to_roads) לתוואי מסודר וב-[`nearest-roads`](../reference/tools#maps_nearest_roads) לנקודות עצמאיות. שמרו את `originalIndex`; אינטרפולציה מוסיפה נקודות שייתכן שאין להן אינדקס. מגבלות מהירות אינן מוחזרות.

```sh
mcp-gmap-rs nearest-roads --points '[{"latitude":48.8584,"longitude":2.2945}]'
mcp-gmap-rs static-map --center '48.8584,2.2945' --zoom 14 --output area.png
```

[מפרט הכלים](../reference/tools) מפרט גבולות מדויקים, שדות נדרשים, ברירות מחדל ופורמטי תוצאה לכל פעולה.

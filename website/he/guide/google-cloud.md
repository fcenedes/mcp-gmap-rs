# הגדרת Google Cloud

קובץ ההפעלה מאמת **קריאות יוצאות ל-Google** באמצעות מפתח API או פרטי OAuth 2.0. `MCP_HTTP_TOKEN` מאמת בנפרד לקוחות שמתחברים לשרת MCP HTTP שלכם; בחירה ב-Google OAuth אינה מחליפה אותו.

## הפעלת שירותים וחיוב {#enable-services-and-billing}

ב[מסוף Google Cloud](https://console.cloud.google.com/), בחרו או צרו פרויקט וקשרו חשבון חיוב פעיל תחת **Billing**. הפעילו ממשקים נדרשים תחת **APIs & Services → Library**. הפעלת API והתרתו בהגבלות מפתח הן פעולות נפרדות. קריאות עשויות להיות כרוכות בתשלום.

הטבלה הבאה מתארת את נקודות הקצה המשמשות בגרסה זו:

| Google API | שירות להפעלה | אימות בשימוש כאן |
| --- | --- | --- |
| [Places API (New)](https://developers.google.com/maps/documentation/places/web-service/get-api-key) | `places.googleapis.com` | מפתח API או OAuth |
| [Routes API](https://developers.google.com/maps/documentation/routes/get-api-key) | `routes.googleapis.com` | מפתח API או OAuth |
| [Weather API](https://developers.google.com/maps/documentation/weather/get-api-key) | `weather.googleapis.com` | מפתח API או OAuth |
| [Air Quality API](https://developers.google.com/maps/documentation/air-quality/get-api-key) | `airquality.googleapis.com` | מפתח API או OAuth |
| [Address Validation API](https://developers.google.com/maps/documentation/address-validation/get-api-key) | `addressvalidation.googleapis.com` | מפתח API או OAuth |
| [Places Aggregate API](https://developers.google.com/maps/documentation/places-aggregate/get-api-key) | `areainsights.googleapis.com` | מפתח API או OAuth |
| [Pollen API](https://developers.google.com/maps/documentation/pollen/get-api-key) | `pollen.googleapis.com` | מפתח API |
| [Roads API](https://developers.google.com/maps/documentation/roads/get-api-key) | `roads.googleapis.com` | מפתח API |
| [Geocoding API v3](https://developers.google.com/maps/documentation/geocoding/guides-v3/get-api-key) | `geocoding-backend.googleapis.com` | מפתח API |
| [Elevation API](https://developers.google.com/maps/documentation/elevation/get-api-key) | `elevation-backend.googleapis.com` | מפתח API |
| [Time Zone API](https://developers.google.com/maps/documentation/timezone/get-api-key) | `timezone-backend.googleapis.com` | מפתח API |
| [Maps Static API](https://developers.google.com/maps/documentation/maps-static/get-api-key) | `static-maps-backend.googleapis.com` | מפתח API |

להפעלת כל שנים-עשר השירותים עם חשבון מנהל המחובר ל-`gcloud`:

```sh
export PROJECT_ID='your-project-id'
gcloud services enable --project="$PROJECT_ID" \
  places.googleapis.com routes.googleapis.com \
  weather.googleapis.com airquality.googleapis.com \
  geocoding-backend.googleapis.com elevation-backend.googleapis.com \
  timezone-backend.googleapis.com static-maps-backend.googleapis.com \
  addressvalidation.googleapis.com areainsights.googleapis.com \
  pollen.googleapis.com roads.googleapis.com
```

כלי CLI בשם `directions` ו-`distance-matrix` משתמשים ב-**Routes API**, ולכן אינם דורשים את ממשקי Directions ו-Distance Matrix הישנים. הפרויקט עדיין משתמש ב-Geocoding v3; [תמיכת OAuth ב-Geocoding v4 של Google](https://developers.google.com/maps/documentation/geocoding/geocoding-v4-overview) אינה משנה את האימות של קריאות v3. כלים משולבים עשויים לקרוא לכמה ממשקים. פתרון שם מקום או מנגנון גיבוי עשויים לדרוש Geocoding גם כשהפעולה המרכזית משתמשת ב-Places או Routes. השתמשו בקואורדינטות כדי להימנע מתלות זו, או השאירו מפתח מוגבל לנקודות התומכות רק במפתח.

## יצירת מפתח API מוגבל {#create-a-restricted-api-key}

1. פתחו [Google Maps Platform → Keys & Credentials](https://console.cloud.google.com/google/maps-apis/credentials) בפרויקט הנבחר ובחרו **Create credentials → API key**.
2. בחרו רק בממשקים מהטבלה שהמופע ישתמש בהם. למפתח קיים ערכו את הגבלות ה-API ושמרו.
3. הגדירו את הגבלת היישום ל-**IP addresses** עבור יישום צד שרת זה. התירו את כתובות היציאה הציבוריות ש-Google תראה. כתובות Pod או כתובות פרטיות של Kubernetes בדרך כלל אינן כתובת המקור הציבורית; Cloud NAT קבוע או כתובת יציאה אחרת הופכים את ההגבלה לצפויה.
4. שמרו את המפתח ב-`.env` מוחרג, בהזרקת סביבה או ב-Kubernetes Secret. ראו [הנחיות Google להגבלת מפתחות](https://developers.google.com/maps/api-security-best-practices#restrict-your-api-keys).

דוגמת `.env`:

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

`api-key` הוא ברירת המחדל. ציון נתיב לחשבון שירות בלבד אינו משנה את מצב האימות. גילוי אינו פונה ל-Google; השתמשו בקריאה אמיתית לאימות חיוב הפרויקט, הפעלת ממשקים והגבלות:

```sh
mcp-gmap-rs geocode --address 'Paris, France'
```


## הצעדים הבאים {#next-steps}

עבור JSON של חשבון שירות, ADC, התחזות מקומית וזהות GKE, עקבו אחר [אימות זהות](./authentication). לאחר מכן בחרו [CLI](./cli) או [MCP](./mcp). [מפרט הכלים](../reference/tools) מציין את דרישות פרטי הגישה של כל כלי.

# אימות זהות

הגדירו את פרטי הגישה היוצאים ל-Google בנפרד מפרטי הגישה של לקוחות MCP HTTP. מצב ברירת המחדל הוא מפתח API; הגדרת נתיב לחשבון שירות לבדה אינה מפעילה OAuth.

## תמיכה במפתח API וב-OAuth {#api-key-and-oauth-coverage}

| משפחת נקודות קצה | מפתח API | OAuth בקובץ הפעלה זה |
| --- | --- | --- |
| Places (New), Routes, Weather, Air Quality, Address Validation, Places Aggregate | כן | כן |
| Geocoding v3, Elevation, Time Zone, Maps Static, Pollen, Roads | כן | לא |

פתרון מיקומים לפי שם וקריאות משולבות עשויים לדרוש מפתח גם כשהפעולה העיקרית תומכת ב-OAuth. השתמשו בקואורדינטות מדויקות כדי למנוע קידוד גאוגרפי מיותר. כשלי OAuth לעולם אינם עוברים בשקט למפתח API. ראו [הגדרת Google Cloud](./google-cloud) להפעלת השירותים המדויקים ולהגבלת מפתח.

## OAuth באמצעות חשבון שירות {#service-account-oauth}


במצב OAuth, בקשות Places (New), Routes, Weather, Air Quality, Address Validation ו-Places Aggregate מקבלות **אסימון גישה** של OAuth וכותרת `X-Goog-User-Project`. הפרויקט מבקש את ההרשאה `https://www.googleapis.com/auth/cloud-platform` המתועדת עבור ממשקי API אלה. ספריית האימות של Google ל-Rust משיגה, שומרת במטמון ומרעננת את האסימונים. `GOOGLE_MAPS_TIMEOUT` מגביל גם את אתחול פרטי הגישה ואת השגת האסימון. בזרימת חשבון השירות אין צורך להדביק אסימון ידנית או להגדיר מסך הסכמה של OAuth לאפליקציה. ראו [הנחיות OAuth בצד השרת של Maps](https://developers.google.com/maps/api-security-best-practices#use-oauth-for-server-side-apps).

צרו חשבון שירות ייעודי והעניקו לו הרשאה לצרוך שירותים בפרויקט המשמש לחיוב ולמכסות של Maps:

```sh
export PROJECT_ID='your-project-id'
export SA_EMAIL="mcp-gmap@${PROJECT_ID}.iam.gserviceaccount.com"
gcloud iam service-accounts create mcp-gmap --project="$PROJECT_ID" \
  --display-name='MCP Google Maps'
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:${SA_EMAIL}" \
  --role='roles/serviceusage.serviceUsageConsumer'
```

התפקיד Service Usage Consumer כולל את `serviceusage.services.use` לצורכי מכסה וחיוב. הוא אינו מעניק אפשרות להפעיל שירותים או ליצור מפתחות; שלבי הגדרה אלה דורשים מנהל בעל הרשאות נפרדות. הפעילו את ממשקי Maps בפרויקט המכסה לפי [מדריך Google Cloud](./google-cloud). ראו [הרשאות Service Usage](https://docs.cloud.google.com/service-usage/docs/access-control).

בחרו אחד ממקורות פרטי הגישה שבהמשך. אותן הגדרות פועלות עם CLI, stdio ו-HTTP. סדר הקדימויות הרגיל הוא קובץ, אחריו סביבה ולבסוף אפשרויות שורת הפקודה.

### א. מפתח JSON מותקן של חשבון שירות {#a-mounted-service-account-json-key}

אם הארגון מאפשר מפתחות JSON, פתחו **IAM & Admin → Service Accounts**, בחרו בחשבון ואז **Keys → Add key → Create new key → JSON**. שמרו אותו מחוץ לעותק המקור. לחלופין:

```sh
mkdir -p "$HOME/.config/mcp-gmap-rs"
gcloud iam service-accounts keys create \
  "$HOME/.config/mcp-gmap-rs/service-account.json" \
  --iam-account="$SA_EMAIL" --project="$PROJECT_ID"
chmod 600 "$HOME/.config/mcp-gmap-rs/service-account.json"
export GOOGLE_APPLICATION_CREDENTIALS="$HOME/.config/mcp-gmap-rs/service-account.json"
export GOOGLE_CLOUD_QUOTA_PROJECT="$PROJECT_ID"
export GOOGLE_MAPS_AUTH_MODE=oauth
```

עבור קובץ מותקן, תוכן `.env` המקביל הוא:

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
# Optional, for Geocoding v3 / Elevation / Time Zone / Static Maps / Pollen / Roads:
# GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

הגדירו נתיב לפרטי גישה רק כשמשתמשים בקובץ JSON קריא; אל תגדירו מחרוזת ריקה או את תוכן ה-JSON עצמו. נתיבים יחסיים נפתרים מתוך תיקיית העבודה. ערכי הקובץ משמשים ישירות בלי לשנות את סביבת התהליך. הטוען מקבל גם תצורות ADC מסוג `external_account`, `impersonated_service_account` ו-`authorized_user`. הפעילו מחדש את התהליך לאחר שינוי קובץ פרטי הגישה.

מפתחות חשבון שירות הם פרטי גישה ארוכי טווח. העדיפו אפשרויות ללא מפתח כשאפשר; שמרו מפתחות JSON מחוץ ל-Git ולשכבות תמונה, והחליפו או מחקו מפתחות שאינם בשימוש. Google מתעדת [יצירת מפתחות ומדיניות ארגונית](https://docs.cloud.google.com/iam/docs/keys-create-delete).

### ב. Application Default Credentials ללא מפתח JSON {#b-application-default-credentials-without-a-json-key}

השאירו את `GOOGLE_APPLICATION_CREDENTIALS` לא מוגדר הן בסביבת התהליך והן בקובץ dotenv שנבחר או ב-Secret/ConfigMap של Kubernetes. פקודת `unset` במעטפת לבדה אינה מסירה ערך משכבת הקובץ; מחקו את הערך במקום להגדיר נתיב ריק. לאחר מכן הגדירו:

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
```

ספריית האימות הרשמית משתמשת ב-[Application Default Credentials (ADC)](https://docs.cloud.google.com/docs/authentication/application-default-credentials): פרטי ADC מקומיים אם קיימים, אחרת חשבון השירות או זהות עומס העבודה המחוברים דרך שרת המטא-נתונים. גילוי פרטי הגישה מתבצע רק כשצריך: `tools`, `schema` והעזרה פועלים בלי לקבל אסימון. קריאת OAuth דורשת פרויקט מכסה ופרטי גישה תקינים.

לפיתוח מקומי באמצעות התחזות לחשבון שירות, הפעילו IAM Credentials API והעניקו לחשבון המפתח את התפקיד Token Creator בחשבון השירות, ואז צרו פרטי ADC מקומיים:

```sh
gcloud services enable iamcredentials.googleapis.com --project="$PROJECT_ID"
gcloud iam service-accounts add-iam-policy-binding "$SA_EMAIL" \
  --project="$PROJECT_ID" --member='user:you@example.com' \
  --role='roles/iam.serviceAccountTokenCreator'
unset GOOGLE_APPLICATION_CREDENTIALS
gcloud auth application-default login --impersonate-service-account="$SA_EMAIL"
export GOOGLE_MAPS_AUTH_MODE=oauth
export GOOGLE_CLOUD_QUOTA_PROJECT="$PROJECT_ID"
```

כך משתמשים באסימון קצר טווח של חשבון השירות לקריאות Maps בלי להוריד את המפתח הפרטי שלו. ראו [התחזות לחשבון שירות](https://docs.cloud.google.com/iam/docs/service-account-impersonation).

### ג. Kubernetes וזהות עומס עבודה ב-GKE {#c-kubernetes-and-gke-workload-identity}

ב-Kubernetes כללי, התקינו את קובץ ה-JSON כקובץ Secret והגדירו את שלושת משתני OAuth שבאפשרות א. הקובץ חייב להיות קריא ל-UID 10001 של הקונטיינר. התקינו אותו לקריאה בלבד ושמרו את `MCP_HTTP_TOKEN` ב-Secret נפרד בזמן הריצה. [מדריך הפריסה](./deployment) מסביר את ה-Deployment הבסיסי.

ב-GKE העדיפו [Workload Identity Federation for GKE](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/workload-identity) עם Kubernetes ServiceAccount המקושר לחשבון שירות של IAM. הפעילו זאת קודם באשכול ובמאגר הצמתים (Autopilot כבר מפעיל זאת), ואז:

```sh
gcloud services enable iamcredentials.googleapis.com --project="$PROJECT_ID"
kubectl create serviceaccount mcp-gmap-rs --namespace default
gcloud iam service-accounts add-iam-policy-binding "$SA_EMAIL" \
  --project="$PROJECT_ID" --role='roles/iam.workloadIdentityUser' \
  --member="serviceAccount:${PROJECT_ID}.svc.id.goog[default/mcp-gmap-rs]"
kubectl annotate serviceaccount mcp-gmap-rs --namespace default \
  "iam.gke.io/gcp-service-account=${SA_EMAIL}"
kubectl patch deployment mcp-gmap-rs --namespace default --type=merge \
  -p '{"spec":{"template":{"spec":{"serviceAccountName":"mcp-gmap-rs"}}}}'
kubectl set env deployment/mcp-gmap-rs --namespace default \
  GOOGLE_MAPS_AUTH_MODE=oauth GOOGLE_CLOUD_QUOTA_PROJECT="$PROJECT_ID"
```

פקודות אלה מניחות שהאשכול וחשבון IAM נמצאים ב-`PROJECT_ID` ושה-Deployment הבסיסי ב-`default`. התאימו את שמות הפרויקט ומרחב השמות להתקנה. אל תגדירו `GOOGLE_APPLICATION_CREDENTIALS` בזרימת מטא-נתונים זו. ה-Pod מקבל אסימונים משרת המטא-נתונים של GKE. מחוץ ל-GKE אפשר להשתמש בתצורת `external_account` ובאסימון זהות מוקרן של הספק במסגרת Workload Identity Federation של Google.

להפעלה ב-OAuth בלבד, הסירו את מפתח Google API מה-Secret בזמן הריצה והפעילו רק כלים וקלטים שקריאותיהם נשארות בממשקים התומכים ב-OAuth. השאירו מפתח כשמשתמשים בנקודות הקצה התומכות רק במפתח שבטבלה או בתהליכים משולבים הזקוקים להן. כשלי OAuth מדווחים במפורש ואינם עוברים בשקט למפתח.

## אימות ופתרון בעיות {#verify-and-troubleshoot}

```sh
mcp-gmap-rs --auth-mode oauth search-places --query 'coffee in Paris'
mcp-gmap-rs --auth-mode oauth weather --latitude 48.8584 --longitude 2.2945
```

אלה קריאות אמיתיות העשויות להיות כרוכות בתשלום. בדקו את המצב האפקטיבי ואת פרויקט המכסה בבדיקה; חיוב OAuth משתמש ב-`GOOGLE_CLOUD_QUOTA_PROJECT`, ואילו קריאות עם מפתח משתמשות בפרויקט המפתח. הפרויקטים עשויים להיות שונים.

השתמשו ב-`mcp-gmap-rs doctor` לבדיקות תצורה לא מקוונות, או ב-`doctor --live` לבדיקות API חיות הכרוכות בתשלום. [מדריך הייצור](./production) מתאר את הדוח וסינון הכלים.

לאחר ייצוא משתני OAuth, הריצו את שש בדיקות הקבלה החיות האופציונליות:

```sh
cargo test --locked --test live live_oauth_ -- --ignored --test-threads=1
```

בדיקות אלה מסירות במפורש את מפתח ה-API מכל תהליך משנה, ולכן הצלחה בודקת בפועל OAuth. הן מכסות Places, Routes, Weather, Air Quality, Address Validation ו-Places Aggregate ודורשות שממשקי API אלה יופעלו. הן אינן טוענות `.env`; ייצאו את הגדרות OAuth או הריצו בעומס עבודה מוגדר. בדיקות רגילות ו-CI משתמשות בנתוני דמה מקומיים של אסימונים וממשקים, ואינן דורשות חשבון שירות או חיבור ל-Google.

| תסמין | בדיקה |
| --- | --- |
| `GOOGLE_MAPS_API_KEY is required` | הפעולה מגיעה לנקודת קצה התומכת רק במפתח, או שנבחר מצב מפתח API. |
| אימות OAuth נכשל | בדקו נתיב וסוג פרטי גישה, תוקף מפתח, קישור זהות או התחזות, IAM Credentials API אם רלוונטי, קישוריות לנקודת האסימונים ודיוק שעון. שגיאות היישום אינן כוללות פרטי גישה או אסימונים. |
| `GOOGLE_CLOUD_QUOTA_PROJECT is required` | הגדירו מזהה או מספר של פרויקט צרכן Maps לקריאות OAuth. |
| Google `403` / API לא פעיל | הפעילו את השירות המדויק בפרויקט הצרכן ובדקו חיוב והרשאת Service Usage. |
| דחיית IP במפתח API | התאימו הגבלות לכתובת היציאה הציבורית של השרת או האשכול. |
| פג תוקף ההמתנה לאסימון | אפשרו גישה לנקודות האסימון, STS ו-IAM של Google או לשרת המטא-נתונים של עומס העבודה; בדקו `GOOGLE_MAPS_TIMEOUT`. |
| HTTP `401` משרת MCP זה | שלחו את `MCP_HTTP_TOKEN` שלו; אסימוני חשבון שירות של Google מאמתים חיבור יוצא נפרד. |


## שירותים נוספים ומגבלות {#expanded-services}

Solar ו־Map Tiles תומכים גם ב־OAuth. שירותי Datasets ו־Route Optimization דורשים OAuth והרשאות IAM לכל פעולה. Street View Publish דורש קובץ authorized_user שנבחר במפורש והסכמת המשתמש להיקף streetviewpublish.

| שירות | API | אימות |
| --- | --- | --- |
| Geolocation API | `geolocation.googleapis.com` | מפתח API |
| Solar API | `solar.googleapis.com` | מפתח API / OAuth |
| Aerial View API | `aerialview.googleapis.com` | מפתח API |
| Map Tiles API | `tile.googleapis.com` | מפתח API / OAuth |
| Street View Static API | `street-view-image-backend.googleapis.com` | מפתח API |
| Maps Platform Datasets API | `mapsplatformdatasets.googleapis.com` | OAuth / IAM |
| Route Optimization API | `routeoptimization.googleapis.com` | OAuth / IAM |
| Street View Publish API | `streetviewpublish.googleapis.com` | OAuth של משתמש שנתן הסכמה; חשבונות שירות נדחים |

Speed Limits דורש זכאות מוגבלת ל־Asset Tracking. Aerial View מוגבל לכתובות נתמכות בארצות הברית ואוסר הורדה או שמירה במטמון של סרטונים. Map Tiles מיועד להצגה בלבד; ניתוח אוטומטי וחילוץ נתונים אסורים. כלי Weather לדקות ולמפות הם ניסיוניים, עם מטמון לשעה לכל היותר ומגבלות שימוש נוספות. תיעוד כל כלי מפרט תנאים, ייחוס ומגבלות.

[Google: Speed Limits](https://developers.google.com/maps/documentation/roads/speed-limits), [Map Tiles](https://developers.google.com/maps/documentation/tile/policies), [Aerial View](https://developers.google.com/maps/documentation/aerial-view/policies), [Street View Publish](https://developers.google.com/streetview/publish/authorizing).

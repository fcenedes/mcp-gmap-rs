# פריסה

ליצירת מפתח Google API, להפעלת הממשקים ול-OAuth של חשבון שירות, עקבו אחר [מדריך Google Cloud](./google-cloud). אותן שכבות תצורה פועלות בקונטיינרים. OAuth יכול להשתמש בקובץ JSON מותקן או בזהות ADC של עומס העבודה; פרטי הגישה מסופקים בזמן הריצה.

הורידו או שכפלו את [מאגר המקור](https://github.com/fcenedes/mcp-gmap-rs), ובנו מתיקיית השורש:

```sh
docker build -t mcp-gmap-rs:local .
```

ה-Dockerfile הרב-שלבי מקמפל עם Rust 1.94.1 וקובץ הנעילה. סביבת הריצה כוללת קובץ הפעלה, תעודות CA של Debian ורישיונות, פועלת כ-UID/GID 10001 ותומכת במערכת קבצים לקריאה בלבד. הקשר הבנייה המבוסס על רשימת היתר אינו כולל קובצי `.env` מקומיים, נתוני Git או פלט בנייה. הבנייה אינה דורשת פרטי גישה. BuildKit שומר הורדות Cargo ופלט בנייה במטמון לבניות הבאות.

אותו Dockerfile נבנה באופן טבעי ב-Linux amd64 ו-arm64. כדי לבנות את שניהם ולדחוף למאגר התמונות שלכם באמצעות בונה Buildx מוגדר:

```sh
docker buildx build --platform linux/amd64,linux/arm64 \
  -t registry.example.com/your-team/mcp-gmap-rs:YOUR_VERSION --push .
```

פרסום למאגר תמונות אינו מוגדר אוטומטית. החליפו שמות תמונות וגרסאות בתמונה שבניתם או פרסמתם.

## הפעלת HTTP, CLI או stdio {#run-http-cli-or-stdio}

ברירת המחדל של התמונה היא `serve http` ב-`0.0.0.0:3000`. ספקו אסימון bearer בזמן הריצה, למשל באותו פורמט `.env` המופיע ב-`.env.example`:

```sh
docker run --rm --read-only --cap-drop ALL --security-opt no-new-privileges \
  --env-file .env -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

ודאו ש-`.env` כולל `MCP_HTTP_TOKEN`, הנדרש כי הקונטיינר מאזין מחוץ ל-loopback. קריאות Google משתמשות ב-`GOOGLE_MAPS_API_KEY` או בהגדרות OAuth ובהתקנות פרטי הגישה שבמדריך, לפי הכלים המופעלים. הלקוחות משתמשים ב-`http://127.0.0.1:3000/mcp` וב-`Authorization: Bearer <token>`. `GET /health` אינו דורש אסימון ואינו פונה ל-Google.

`--env-file` של Docker מזריק משתני סביבה. כדי להפעיל במקום זאת את טוען הקבצים של היישום, התקינו קובץ קריא ובחרו בו:

```sh
docker run --rm --read-only \
  --mount type=bind,src="$(pwd)/.env",dst=/etc/mcp-gmap/runtime.env,readonly \
  -e GOOGLE_MAPS_LANGUAGE=fr -p 127.0.0.1:3000:3000 \
  mcp-gmap-rs:local --env-file /etc/mcp-gmap/runtime.env serve http
```

הסביבה גוברת על ערכי הקובץ המותקן, ואפשרויות CLI גוברות על שניהם. גם `MCP_HTTP_LISTEN=0.0.0.0:3000` שבתמונה הוא משתנה סביבה; שנו אותו עם `-e MCP_HTTP_LISTEN=...` או `--listen` לפי הצורך. לשימוש במשתני סביבה בלבד, העבירו `--no-env-file serve http`.

החליפו את פקודת התמונה לשימוש ב-CLI או ב-stdio:

```sh
docker run --rm mcp-gmap-rs:local tools
docker run --rm --env-file .env mcp-gmap-rs:local geocode --address Paris
docker run --rm -i --env-file .env mcp-gmap-rs:local serve stdio
```

השתמשו ב-`-i` עבור stdio בלי להקצות TTY. לכתיבת קובצי פלט CLI, התקינו תיקייה הניתנת לכתיבה בידי UID 10001 והשתמשו ב-`--output` בתוכה.

## Kubernetes {#kubernetes}

[קובץ Kubernetes הבסיסי](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/kubernetes.yaml) מספק הגדרות לא סודיות דרך ConfigMap ופרטי גישה דרך Secret קיים בשם `mcp-gmap-rs`. צרו Secret זה מקובץ מקומי המכיל `GOOGLE_MAPS_API_KEY` ו-`MCP_HTTP_TOKEN` (הקובץ אינו נשמר ב-Git):

```sh
kubectl create secret generic mcp-gmap-rs --from-env-file=/secure/credentials.env
```

החליפו את `mcp-gmap-rs:local` ב-Deployment בתמונה מהמאגר שלכם, או טענו את התמונה המקומית לאשכול הפיתוח. לאחר מכן החילו את הדוגמה:

```sh
kubectl apply -f deploy/kubernetes.yaml
kubectl rollout status deployment/mcp-gmap-rs
kubectl port-forward service/mcp-gmap-rs 3000:3000
```

הדוגמה מיועדת למרחב השמות `default`. התאימו שמות DNS של שירותים כשמשתמשים במרחב שמות אחר או ב-ingress. שירות ClusterIP פרטי לאשכול; השתמשו ב-ingress או ב-proxy עם TLS עבור לקוחות מרוחקים, תוך העברת Host הרשום ב-`MCP_HTTP_ALLOWED_HOSTS`, הרשאות וכותרות MCP/SSE. הוסיפו מקורות דפדפן מפורשים, כולל פורטים, לפי הצורך.

הפריסה משתמשת ב-`--no-env-file` כי כל התצורה מוזרקת דרך `envFrom`. לחלופין, התקינו Secret המכיל קובץ dotenv, הסירו את האפשרות והגדירו `args: ["--env-file", "/etc/mcp-gmap/runtime.env", "serve", "http"]`. הקובץ המוקרן חייב להיות קריא ל-UID 10001; משתני סביבה מ-Secrets או ConfigMaps עדיין גוברים עליו. ראו בתיעוד Kubernetes על [הזרקת סביבה](https://kubernetes.io/docs/tasks/inject-data-application/define-environment-variable-container/) ו[אמצעי אחסון Secret](https://kubernetes.io/docs/concepts/configuration/secret/).

תצורת היישום נקראת בהפעלה. הפעילו מחדש Pods לאחר שינוי ערכי Secret/ConfigMap; קובצי TLS מובנה תומכים גם בטעינה מחדש ב-SIGHUP. בדיקות משתמשות ב-`/health` לזמינות HTTP; Pod בריא אינו מוכיח ש-Google API מופעל או שפרטי הגישה מורשים. קריאות Google דורשות HTTPS יוצא ו-DNS תקין. הגבלות המפתח חייבות להתיר את כתובת היציאה הציבורית של האשכול; OAuth דורש גישה לנקודות האסימון והזהות.

השאירו עותק אחד עבור מאגר סשני MCP ברירת המחדל שבזיכרון. מספר עותקים דורש שיוך סשנים כדי שכל סשן יגיע ל-Pod שלו; הפעלות מחדש מדורגות מאבדות סשנים והלקוחות חייבים להתחבר מחדש. בקשות ומגבלות המשאבים בדוגמה הן ערכי התחלה להתאמה לעומס העבודה.

SIGTERM מפעיל כיבוי מסודר של MCP/HTTP; Kubernetes מאפשר 30 שניות בדוגמה. התמונה פועלת גם ללא capabilities וללא אפשרות להסלמת הרשאות.

## HTTPS, מדדים ומגבלות בייצור {#production-https-metrics-and-limits}

[מדריך הייצור](./production) מכסה TLS/mTLS מובנה, טעינת תעודות, שער HTTPS אופציונלי `deploy/gateway.yaml`, מדדים, תקציבי בקשות, הגבלת קצב HTTP, יומני JSON ובדיקות `doctor` לא מקוונות וחיות. העבירו את אותן אפשרויות TLS אחרי שם התמונה, או הזריקו `MCP_TLS_CERT`, `MCP_TLS_KEY` ו-`MCP_TLS_CLIENT_CA` האופציונלי עם קובצי PEM לקריאה בלבד. שמרו מפתחות אמיתיים מחוץ לתמונה. ConfigMap ברירת המחדל מגדיר מגבלות בקשות ורישום JSON; מדדים כבויים עד הפעלה מפורשת. שירות HTTP ברירת המחדל מיועד למקטע הפרטי בין השער ל-Pod.

## פרטי גישה של OAuth בקונטיינר {#oauth-credentials-in-a-container}

עבור קובץ JSON של חשבון שירות, התקינו אותו לקריאה בלבד והגדירו את הנתיב בתוך הקונטיינר, לא את הנתיב במארח:

```sh
docker run --rm --read-only --env-file /secure/oauth.env \
  --mount type=bind,src=/secure/service-account.json,dst=/credentials/service-account.json,readonly \
  -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

קובץ סביבת OAuth מגדיר `GOOGLE_MAPS_AUTH_MODE=oauth`, `GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json`, `GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id` ו-`MCP_HTTP_TOKEN`. UID 10001 חייב לקרוא את הקובץ המותקן. השאירו מפתח API רק לכלים הדורשים נקודות קצה התומכות במפתח בלבד. ב-GKE השתמשו ב[הוראות זהות עומס העבודה](./authentication#c-kubernetes-and-gke-workload-identity) במקום להתקין מפתח ארוך טווח.

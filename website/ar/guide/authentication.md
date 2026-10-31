# المصادقة

اضبط بيانات اعتماد Google الصادرة بمعزل عن بيانات اعتماد عملاء MCP HTTP. الوضع الافتراضي هو مفتاح API؛ وتعيين مسار حساب خدمة وحده لا يفعّل OAuth.

## تغطية مفتاح API وOAuth {#api-key-and-oauth-coverage}

| مجموعة نقاط النهاية | مفتاح API | OAuth في هذا الملف التنفيذي |
| --- | --- | --- |
| Places (New), Routes, Weather, Air Quality, Address Validation, Places Aggregate | نعم | نعم |
| Geocoding v3, Elevation, Time Zone, Maps Static, Pollen, Roads | نعم | لا |

قد يتطلب تحويل المواقع المسماة والاستدعاءات المركبة مفتاحًا حتى عندما تدعم العملية الأساسية OAuth. استخدم إحداثيات دقيقة لتجنب الترميز الجغرافي غير الضروري. لا تتحول إخفاقات OAuth بصمت إلى مفتاح API. راجع [إعداد Google Cloud](./google-cloud) لتفعيل الخدمات المطلوبة وتقييد المفتاح.

## OAuth باستخدام حساب خدمة {#service-account-oauth}


في وضع OAuth، تتلقى طلبات Places (New) وRoutes وWeather وAir Quality وAddress Validation وPlaces Aggregate **رمز وصول** OAuth وترويسة `X-Goog-User-Project`. يطلب هذا المشروع النطاق `https://www.googleapis.com/auth/cloud-platform` الموثّق لهذه الواجهات. تحصل مكتبة مصادقة Google لـ Rust على الرموز وتخزّنها مؤقتًا وتجددها. يحدّ `GOOGLE_MAPS_TIMEOUT` أيضًا من مدة تهيئة بيانات الاعتماد والحصول على الرمز. لا تحتاج آلية حساب الخدمة إلى لصق رمز وصول يدويًا أو شاشة موافقة OAuth للتطبيق. راجع [إرشادات OAuth للخوادم في Maps](https://developers.google.com/maps/api-security-best-practices#use-oauth-for-server-side-apps).

أنشئ حساب خدمة مخصصًا وامنحه إذن استهلاك الخدمات في المشروع المستخدم لفوترة Maps وحصصه:

```sh
export PROJECT_ID='your-project-id'
export SA_EMAIL="mcp-gmap@${PROJECT_ID}.iam.gserviceaccount.com"
gcloud iam service-accounts create mcp-gmap --project="$PROJECT_ID" \
  --display-name='MCP Google Maps'
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:${SA_EMAIL}" \
  --role='roles/serviceusage.serviceUsageConsumer'
```

يتضمن دور Service Usage Consumer الإذن `serviceusage.services.use` للحصص والفوترة. لكنه لا يمنح إمكانية تفعيل الخدمات أو إنشاء المفاتيح؛ وتحتاج خطوات الإعداد هذه إلى مسؤول مخوّل بصورة مستقلة. فعّل واجهات Maps في مشروع الحصة باستخدام [دليل إعداد Google Cloud](./google-cloud). راجع [أذونات Service Usage](https://docs.cloud.google.com/service-usage/docs/access-control).

اختر أحد مصادر بيانات الاعتماد أدناه. تعمل الإعدادات نفسها مع CLI وstdio وHTTP. تتبع الإعدادات ترتيب الأولوية المعتاد: الملف ثم البيئة ثم الخيار.

### أ. مفتاح JSON لحساب خدمة مُركّب {#a-mounted-service-account-json-key}

إذا سمحت مؤسستك بمفاتيح JSON، افتح **IAM & Admin → Service Accounts**، واختر الحساب، ثم **Keys → Add key → Create new key → JSON**. احفظ الملف خارج نسخة المصدر. أو استخدم:

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

بالنسبة إلى ملف مُركّب، يكون ملف `.env` المكافئ:

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
# Optional, for Geocoding v3 / Elevation / Time Zone / Static Maps / Pollen / Roads:
# GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

عيّن مسار بيانات الاعتماد فقط عند استخدام ملف JSON قابل للقراءة؛ ولا تضبطه كسلسلة فارغة أو كمحتوى JSON نفسه. تُحل المسارات النسبية انطلاقًا من دليل العمل. تُستخدم قيم الملف مباشرة من دون تعديل بيئة العملية. يقبل المحمّل أيضًا إعدادات JSON لـ ADC من الأنواع `external_account` و`impersonated_service_account` و`authorized_user`. أعد تشغيل العملية بعد تغيير ملف بيانات الاعتماد.

مفاتيح حساب الخدمة بيانات اعتماد طويلة العمر. فضّل الخيارات دون مفتاح أدناه متى توفرت؛ وأبقِ مفاتيح JSON خارج Git وطبقات الصور، ودوّر المفاتيح غير المستخدمة أو احذفها. توثّق Google [إنشاء المفاتيح وسياسات المؤسسات](https://docs.cloud.google.com/iam/docs/keys-create-delete).

### ب. بيانات اعتماد التطبيق الافتراضية دون مفتاح JSON {#b-application-default-credentials-without-a-json-key}

اترك `GOOGLE_APPLICATION_CREDENTIALS` غير مضبوط في بيئة العملية وفي ملف dotenv المحدد أو Secret/ConfigMap في Kubernetes. لا يزيل `unset` في الصدفة وحده قيمة موجودة في طبقة الملف؛ احذف المدخل بدل ضبط مسار فارغ. ثم عيّن:

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
```

تستخدم مكتبة المصادقة الرسمية [بيانات اعتماد التطبيق الافتراضية (ADC)](https://docs.cloud.google.com/docs/authentication/application-default-credentials): بيانات ADC المحلية إن وجدت، وإلا حساب الخدمة أو هوية عبء العمل المرتبطة عبر خادم البيانات الوصفية. يُجرى اكتشاف بيانات الاعتماد عند الحاجة؛ تعمل `tools` و`schema` والمساعدة دون جلب رمز. يحتاج استدعاء OAuth إلى مشروع حصة وبيانات اعتماد صالحة.

للتطوير المحلي بانتحال حساب خدمة، فعّل IAM Credentials API وامنح حساب المطوّر دور Token Creator على حساب الخدمة، ثم أنشئ بيانات ADC محلية:

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

يستخدم ذلك رمز حساب خدمة قصير العمر لاستدعاءات Maps دون تنزيل مفتاحه الخاص. راجع [انتحال حساب الخدمة](https://docs.cloud.google.com/iam/docs/service-account-impersonation).

### ج. Kubernetes وهوية عبء العمل في GKE {#c-kubernetes-and-gke-workload-identity}

في Kubernetes العام، ركّب JSON الخاص ببيانات الاعتماد كملف Secret واضبط متغيرات OAuth الثلاثة في الخيار أ. يجب أن يتمكن UID 10001 المستخدم في الحاوية من قراءة الملف. ركّبه للقراءة فقط وأبقِ `MCP_HTTP_TOKEN` في Secret مستقل وقت التشغيل. يشرح [دليل الحاويات](./deployment) النشر الأساسي.

في GKE، فضّل [Workload Identity Federation for GKE](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/workload-identity) مع Kubernetes ServiceAccount مرتبط بحساب خدمة IAM. فعّلها أولًا للعنقود ومجموعة العقد (تكون مفعّلة أصلًا في Autopilot)، ثم:

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

تفترض هذه الأوامر وجود العنقود وحساب خدمة IAM في `PROJECT_ID` ووجود النشر الأساسي في `default`. عدّل أسماء المشروع ومساحة الأسماء لبيئتك. لا تضبط `GOOGLE_APPLICATION_CREDENTIALS` لهذا التدفق؛ تحصل الحجرة على رموز الوصول عبر خادم بيانات GKE الوصفية. خارج GKE، يمكن استخدام إعداد `external_account` مع رمز هوية مُسقط من الموفّر ضمن إعداد Workload Identity Federation لدى Google.

للتشغيل باستخدام OAuth فقط، أزل مفتاح Google API من Secret وقت التشغيل وفعّل فقط الأدوات والمدخلات التي تبقى طلباتها ضمن الواجهات الداعمة لـ OAuth. احتفظ بالمفتاح عند استخدام نقاط النهاية الخاصة بالمفتاح في الجدول أو التدفقات المركبة التي تحتاج إليها. تُبلغ إخفاقات OAuth صراحة ولا تتحول بصمت إلى المفتاح.

## التحقق واستكشاف المشكلات {#verify-and-troubleshoot}

```sh
mcp-gmap-rs --auth-mode oauth search-places --query 'coffee in Paris'
mcp-gmap-rs --auth-mode oauth weather --latitude 48.8584 --longitude 2.2945
```

هذه استدعاءات حقيقية قد تُفرض رسوم عليها. تحقّق من الوضع الفعّال ومشروع الحصة عند الاختبار؛ تستخدم فوترة OAuth المتغير `GOOGLE_CLOUD_QUOTA_PROJECT` بينما تستخدم طلبات المفتاح مشروع المفتاح نفسه. وقد يختلف المشروعان.

استخدم `mcp-gmap-rs doctor` لفحوص الإعدادات غير المتصلة، أو `doctor --live` لمجسات API القابلة للفوترة صراحة. يصف [دليل الإنتاج](./production) التقرير وتصفية الأدوات.

بعد تصدير متغيرات OAuth، شغّل اختبارات القبول الحية الستة الاختيارية باستخدام:

```sh
cargo test --locked --test live live_oauth_ -- --ignored --test-threads=1
```

تزيل هذه الاختبارات مفتاح API صراحة من كل عملية فرعية، لذلك يختبر نجاحها OAuth فعليًا. وهي تغطي Places وRoutes وWeather وAir Quality وAddress Validation وPlaces Aggregate وتتطلب تفعيل واجهاتها. لا تُحمّل `.env`؛ صدّر إعدادات OAuth أو نفّذها في عبء عمل مضبوط. تستخدم الاختبارات المعتادة وCI بدائل محلية للرموز والواجهات، ولا تتطلب حساب خدمة أو اتصالًا بـ Google.

| العَرَض | الفحص |
| --- | --- |
| `GOOGLE_MAPS_API_KEY is required` | العملية المختارة تصل إلى نقطة نهاية خاصة بالمفتاح أو تم اختيار وضع مفتاح API. |
| فشل مصادقة OAuth | افحص مسار ونوع بيانات الاعتماد، وصلاحية المفتاح، وربط الهوية أو الانتحال، وIAM Credentials API عند استخدامها، والاتصال بنقطة الرموز ودقة الساعة. لا تتضمن أخطاء التطبيق بيانات الاعتماد أو الرموز. |
| `GOOGLE_CLOUD_QUOTA_PROJECT is required` | اضبط معرّف أو رقم مشروع استهلاك Maps لاستدعاءات OAuth. |
| Google `403` / الواجهة غير مفعّلة | فعّل الخدمة المحددة في مشروع الاستهلاك، وتحقق من الفوترة وإذن Service Usage. |
| رفض عنوان IP لمفتاح API | طابق القيود مع عنوان الخروج العام للخادم أو العنقود. |
| انتهاء مهلة الرمز | اسمح بالوصول إلى نقاط Google للرموز وSTS وIAM أو خادم بيانات عبء العمل الوصفية، وراجع `GOOGLE_MAPS_TIMEOUT`. |
| HTTP `401` من خادم MCP هذا | أرسل `MCP_HTTP_TOKEN` الخاص به؛ تُصادق رموز حساب خدمة Google اتصالًا صادرًا مستقلًا. |

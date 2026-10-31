# إعداد Google Cloud

يصادق هذا الملف التنفيذي **الطلبات الصادرة إلى Google** باستخدام مفتاح API أو بيانات OAuth 2.0. ويصادق `MCP_HTTP_TOKEN` بصورة مستقلة العملاء المتصلين بخادم MCP HTTP لديك؛ ولا يحل اختيار Google OAuth محله.

## تفعيل الخدمات والفوترة {#enable-services-and-billing}

في [وحدة تحكم Google Cloud](https://console.cloud.google.com/)، اختر مشروعًا أو أنشئه، ثم اربط حساب فوترة نشطًا تحت **Billing**. فعّل الواجهات المطلوبة من **APIs & Services → Library**. تفعيل واجهة والسماح بها في قيود مفتاح API خطوتان منفصلتان. قد تترتب رسوم على الطلبات.

يصف الجدول التالي نقاط النهاية المستخدمة في هذا الإصدار من المشروع:

| واجهة Google API | الخدمة المطلوب تفعيلها | المصادقة المستخدمة هنا |
| --- | --- | --- |
| [Places API (New)](https://developers.google.com/maps/documentation/places/web-service/get-api-key) | `places.googleapis.com` | مفتاح API أو OAuth |
| [Routes API](https://developers.google.com/maps/documentation/routes/get-api-key) | `routes.googleapis.com` | مفتاح API أو OAuth |
| [Weather API](https://developers.google.com/maps/documentation/weather/get-api-key) | `weather.googleapis.com` | مفتاح API أو OAuth |
| [Air Quality API](https://developers.google.com/maps/documentation/air-quality/get-api-key) | `airquality.googleapis.com` | مفتاح API أو OAuth |
| [Address Validation API](https://developers.google.com/maps/documentation/address-validation/get-api-key) | `addressvalidation.googleapis.com` | مفتاح API أو OAuth |
| [Places Aggregate API](https://developers.google.com/maps/documentation/places-aggregate/get-api-key) | `areainsights.googleapis.com` | مفتاح API أو OAuth |
| [Pollen API](https://developers.google.com/maps/documentation/pollen/get-api-key) | `pollen.googleapis.com` | مفتاح API |
| [Roads API](https://developers.google.com/maps/documentation/roads/get-api-key) | `roads.googleapis.com` | مفتاح API |
| [Geocoding API v3](https://developers.google.com/maps/documentation/geocoding/guides-v3/get-api-key) | `geocoding-backend.googleapis.com` | مفتاح API |
| [Elevation API](https://developers.google.com/maps/documentation/elevation/get-api-key) | `elevation-backend.googleapis.com` | مفتاح API |
| [Time Zone API](https://developers.google.com/maps/documentation/timezone/get-api-key) | `timezone-backend.googleapis.com` | مفتاح API |
| [Maps Static API](https://developers.google.com/maps/documentation/maps-static/get-api-key) | `static-maps-backend.googleapis.com` | مفتاح API |

لتفعيل الخدمات الاثنتي عشرة بحساب مسؤول مسجل في `gcloud`:

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

تستخدم أداتا CLI المسمّاتان `directions` و`distance-matrix` واجهة **Routes API**، فلا تلزم واجهتا Directions وDistance Matrix القديمتان. ما زال هذا المشروع يستخدم Geocoding v3 ؛ ولا يغير [دعم OAuth في Geocoding v4 لدى Google](https://developers.google.com/maps/documentation/geocoding/geocoding-v4-overview) مصادقة استدعاءات v3 هذه. قد تستدعي الأدوات المركبة عدة واجهات. وقد يحتاج تحويل أسماء المواقع أو المسار البديل إلى Geocoding حتى لو كانت العملية الأساسية تستخدم Places أو Routes. استخدم الإحداثيات لتجنب هذا الاعتماد أو احتفظ بمفتاح مقيّد لنقاط النهاية الخاصة بالمفتاح.

## إنشاء مفتاح API مقيّد {#create-a-restricted-api-key}

1. افتح [Google Maps Platform → Keys & Credentials](https://console.cloud.google.com/google/maps-apis/credentials) في المشروع المحدد واختر **Create credentials → API key**.
2. اختر فقط الواجهات من الجدول التي سيستخدمها هذا المثيل. للمفتاح الموجود، عدّل قيود API واحفظها.
3. اضبط قيد التطبيق إلى **IP addresses** لتطبيق الخادم هذا. اسمح بعناوين الخروج العامة التي تراها Google. عناوين حجرات Kubernetes والعناوين الخاصة ليست عادة العنوان العام للمصدر؛ ويساعد عنوان Cloud NAT ثابت أو عنوان خروج آخر على ثبات هذا القيد.
4. خزّن المفتاح في `.env` مستثنى أو حقن البيئة أو Kubernetes Secret. راجع [إرشادات Google لتقييد المفاتيح](https://developers.google.com/maps/api-security-best-practices#restrict-your-api-keys).

مثال `.env`:

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

الوضع الافتراضي هو `api-key`. لا يغير توفير مسار حساب خدمة وحده وضع المصادقة. لا يتصل الاكتشاف بـ Google؛ استخدم استدعاءً حقيقيًا للتحقق من فوترة المشروع وتفعيل الواجهات والقيود:

```sh
mcp-gmap-rs geocode --address 'Paris, France'
```


## الخطوات التالية {#next-steps}

لـ JSON حساب الخدمة وADC والانتحال المحلي وهوية GKE، اتبع [المصادقة](./authentication). ثم اختر [CLI](./cli) أو [MCP](./mcp). يحدد [مرجع الأدوات](../reference/tools) متطلبات بيانات اعتماد كل أداة.

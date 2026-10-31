# النشر

لإنشاء مفتاح Google API وتفعيل الواجهات وOAuth لحساب الخدمة، اتبع [دليل إعداد Google Cloud](./google-cloud). تعمل الإعدادات ذات الطبقات نفسها داخل الحاويات. يمكن أن يستخدم OAuth ملف JSON مُركّبًا أو هوية ADC لعبء العمل؛ تُزوّد بيانات الاعتماد وقت التشغيل.

نزّل أو استنسخ [مستودع المصدر](https://github.com/fcenedes/mcp-gmap-rs)، ثم ابنِ من جذره:

```sh
docker build -t mcp-gmap-rs:local .
```

يجمع Dockerfile متعدد المراحل باستخدام Rust 1.94.1 وملف القفل. تتضمن بيئة التشغيل الملف التنفيذي وشهادات CA الخاصة بـ Debian والتراخيص، وتعمل بهوية UID/GID 10001 وتدعم نظام ملفات جذريًا للقراءة فقط. يستبعد سياق البناء المحدد بقائمة سماح ملفات `.env` المحلية وبيانات Git ومخرجات البناء. لا يلزم توفير بيانات اعتماد للبناء. يخزّن BuildKit تنزيلات Cargo ومخرجات البناء مؤقتًا للبناء اللاحق.

يبني Dockerfile نفسه محليًا على Linux amd64 وarm64. لبناء الاثنين ودفعهما إلى سجلك باستخدام منشئ Buildx مُعدّ:

```sh
docker buildx build --platform linux/amd64,linux/arm64 \
  -t registry.example.com/your-team/mcp-gmap-rs:YOUR_VERSION --push .
```

لا يُضبط نشر سجل حاويات تلقائيًا. استبدل أسماء الصور والإصدارات بالصورة التي بنيتها أو نشرتها.

## تشغيل HTTP أو CLI أو stdio {#run-http-cli-or-stdio}

تشغّل الصورة افتراضيًا `serve http` على `0.0.0.0:3000`. وفّر رمز bearer وقت التشغيل، مثلًا بتنسيق `.env` نفسه الموضح في `.env.example`:

```sh
docker run --rm --read-only --cap-drop ALL --security-opt no-new-privileges \
  --env-file .env -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

تأكد من احتواء `.env` على `MCP_HTTP_TOKEN`، وهو مطلوب لأن الحاوية تستمع خارج loopback. تستخدم استدعاءات Google المتغير `GOOGLE_MAPS_API_KEY` أو إعدادات OAuth وتركيب بيانات الاعتماد من دليل الإعداد، بحسب الأدوات المفعّلة. يستخدم العملاء `http://127.0.0.1:3000/mcp` والترويسة `Authorization: Bearer <token>`. لا يحتاج `GET /health` رمزًا ولا يتصل بـ Google.

يحقن خيار Docker المسمى `--env-file` متغيرات البيئة. لاختبار محمّل ملفات التطبيق بدلًا منه، ركّب ملفًا قابلًا للقراءة واختره:

```sh
docker run --rm --read-only \
  --mount type=bind,src="$(pwd)/.env",dst=/etc/mcp-gmap/runtime.env,readonly \
  -e GOOGLE_MAPS_LANGUAGE=fr -p 127.0.0.1:3000:3000 \
  mcp-gmap-rs:local --env-file /etc/mcp-gmap/runtime.env serve http
```

تتجاوز البيئة قيم الملف المُركّب، وتتجاوز خيارات CLI كليهما. المتغير `MCP_HTTP_LISTEN=0.0.0.0:3000` في الصورة قيمة بيئة أيضًا؛ غيّره باستخدام `-e MCP_HTTP_LISTEN=...` أو `--listen` عند الحاجة. لاستخدام متغيرات البيئة فقط، مرّر `--no-env-file serve http`.

تجاوز أمر الصورة لاستخدام CLI أو stdio:

```sh
docker run --rm mcp-gmap-rs:local tools
docker run --rm --env-file .env mcp-gmap-rs:local geocode --address Paris
docker run --rm -i --env-file .env mcp-gmap-rs:local serve stdio
```

استخدم `-i` مع stdio دون تخصيص TTY. عند كتابة ملفات مخرجات CLI، ركّب دليلًا يستطيع UID 10001 الكتابة فيه واستخدم `--output` داخله.

## Kubernetes {#kubernetes}

يوفّر [ملف Kubernetes الأساسي](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/kubernetes.yaml) الإعدادات غير السرية عبر ConfigMap وبيانات الاعتماد عبر Secret موجود باسم `mcp-gmap-rs`. أنشئ هذا Secret من ملف محلي يحتوي `GOOGLE_MAPS_API_KEY` و`MCP_HTTP_TOKEN` (لا يُحفظ الملف في Git):

```sh
kubectl create secret generic mcp-gmap-rs --from-env-file=/secure/credentials.env
```

استبدل `mcp-gmap-rs:local` في Deployment بصورة السجل الخاصة بك، أو حمّل صورتك المحلية إلى عنقود التطوير. ثم طبّق المثال:

```sh
kubectl apply -f deploy/kubernetes.yaml
kubectl rollout status deployment/mcp-gmap-rs
kubectl port-forward service/mcp-gmap-rs 3000:3000
```

يستهدف المثال مساحة الأسماء `default`. عدّل أسماء مضيفي DNS للخدمات عند استخدام مساحة أسماء أخرى أو ingress. خدمة ClusterIP خاصة بالعنقود؛ استخدم ingress أو وكيل TLS للعملاء البعيدين، مع تمرير Host مدرج في `MCP_HTTP_ALLOWED_HOSTS` والتفويض وترويسات MCP/SSE. أضف أصول المتصفح الصريحة، بما فيها المنافذ، عند الحاجة.

يستخدم النشر `--no-env-file` لأن جميع الإعدادات تُحقن عبر `envFrom`. بدلًا من ذلك، ركّب Secret يحتوي ملف dotenv، وأزل ذلك الخيار واضبط `args: ["--env-file", "/etc/mcp-gmap/runtime.env", "serve", "http"]`. يجب أن يستطيع UID 10001 قراءة الملف المُسقط؛ وتبقى متغيرات Secrets أو ConfigMaps متغلبة عليه. راجع توثيق Kubernetes عن [حقن البيئة](https://kubernetes.io/docs/tasks/inject-data-application/define-environment-variable-container/) و[وحدات Secret](https://kubernetes.io/docs/concepts/configuration/secret/).

تُقرأ إعدادات التطبيق عند البدء. أعد تشغيل الحجرات بعد تغيير قيم Secret/ConfigMap هذه؛ وتدعم ملفات TLS الأصلية إعادة تحميل SIGHUP أيضًا. تستخدم المجسات `/health` لإتاحة HTTP؛ ولا تثبت صحة الحجرة تفعيل Google API أو السماح لبيانات اعتمادها. تتطلب طلبات Google اتصال HTTPS صادرًا وDNS يعمل. يجب أن تسمح قيود المفتاح بعنوان الخروج العام للعنقود؛ ويتطلب OAuth الوصول إلى نقاط الرمز والهوية.

أبقِ نسخة واحدة لمخزن جلسات MCP الافتراضي الموجود في الذاكرة. تحتاج النسخ المتعددة إلى تثبيت الجلسات كي تصل كل جلسة إلى حجرتها المالكة؛ وإعادة التشغيل المتدرجة تفقد الجلسات ويتعين على العملاء إعادة الاتصال. طلبات الموارد وحدودها في المثال قيم أولية تُضبط حسب عبء العمل.

يؤدي SIGTERM إلى إيقاف MCP/HTTP بصورة سلسة؛ ويسمح Kubernetes بـ 30 ثانية في المثال. تعمل الصورة أيضًا دون إمكانات مميزة ودون تصعيد امتيازات.

## HTTPS والمقاييس والحدود في الإنتاج {#production-https-metrics-and-limits}

يغطي [دليل الإنتاج](./production) TLS/mTLS الأصلي، وإعادة تحميل الشهادات، وبوابة HTTPS الاختيارية `deploy/gateway.yaml`، والمقاييس، وميزانيات الطلبات، وحدود معدل HTTP، وسجلات JSON، وفحوص `doctor` غير المتصلة والحية. مرّر خيارات TLS نفسها بعد اسم الصورة أو احقن `MCP_TLS_CERT` و`MCP_TLS_KEY` و`MCP_TLS_CLIENT_CA` الاختياري مع ملفات PEM مُركّبة للقراءة فقط. أبقِ كل المفاتيح الحقيقية خارج الصورة. يضبط ConfigMap الافتراضي حدود الطلبات وسجلات JSON؛ وتبقى المقاييس معطلة حتى تفعيلها صراحة. الخدمة النصية الافتراضية مخصصة للوصلة الخاصة بين البوابة والحجرة.

## بيانات اعتماد OAuth داخل الحاوية {#oauth-credentials-in-a-container}

لملف JSON لحساب خدمة، ركّبه للقراءة فقط واضبط المسار داخل الحاوية، وليس مسار المضيف:

```sh
docker run --rm --read-only --env-file /secure/oauth.env \
  --mount type=bind,src=/secure/service-account.json,dst=/credentials/service-account.json,readonly \
  -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

يضبط ملف بيئة OAuth القيم `GOOGLE_MAPS_AUTH_MODE=oauth` و`GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json` و`GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id` و`MCP_HTTP_TOKEN`. يجب أن يستطيع UID 10001 قراءة الملف المُركّب. احتفظ بمفتاح API فقط للأدوات التي تتطلب نقاط نهاية خاصة بالمفتاح. في GKE، اتبع [تعليمات هوية عبء العمل](./authentication#c-kubernetes-and-gke-workload-identity) بدل تركيب مفتاح طويل العمر.

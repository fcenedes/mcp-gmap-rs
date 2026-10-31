# البداية

يشمل هذا الدليل 42 أداة في الشفرة الحالية. تحتوي الحزمة المنشورة v0.3.1 على 23 أداة. ابنِ نسخة المستودع الحالية لاستخدام الإضافات؛ تثبيت v0.3.1 لا يتضمنها.

من دليل نسخة المستودع المحلية:

```sh
cargo install --path . --locked
```

## التثبيت {#install}

تتوفر ملفات جاهزة لـ Linux x86_64 وmacOS Apple Silicon وmacOS Intel في [إصدارات GitHub](https://github.com/fcenedes/mcp-gmap-rs/releases). نزّل أرشيف جهازك، وتحقق منه باستخدام `SHA256SUMS`، وفكّه وضع `mcp-gmap-rs` على `PATH`. تستهدف ملفات Linux الإصدار glibc 2.35 أو أحدث. ملفات macOS غير موقعة ولا موثقة من Apple.

باستخدام Rust 1.88 أو أحدث، ثبّت من crates.io:

```sh
cargo install mcp-gmap-rs --version 0.3.1 --locked
mcp-gmap-rs --version
```

لا يلزم وجود بيئة تشغيل Rust أو Python أو JavaScript بعد تجميع الملف التنفيذي. يحتاج المضيف شهادات CA وDNS ووصول HTTPS صادرًا إلى Google.

## ضبط الوصول {#configure-access}

[أنشئ مفتاحًا مقيّدًا وفعّل الخدمات](./google-cloud)، أو اضبط [OAuth لحساب الخدمة](./authentication). خزّن بيانات الاعتماد في `.env` مستثنى من Git داخل دليل العمل:

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

تتجاوز متغيرات البيئة الملف. أبقِ بيانات الاعتماد خارج التحكم بالمصدر وطبقات الصور. دليل العمل مهم: لا يُبحث عن `.env` في الأدلة الأب.

## الاكتشاف والفحص والاستدعاء {#discover-check-call}

لا يرسل الاكتشاف والتشخيص غير المتصل طلبات إلى Google:

```sh
mcp-gmap-rs tools
mcp-gmap-rs schema search-places
mcp-gmap-rs doctor
```

ثم نفّذ أول استدعاء قد يكون قابلًا للفوترة:

```sh
mcp-gmap-rs search-places --query 'coffee near Gare de Lyon, Paris'
```

النتيجة JSON. احتفظ بمعرّفات الأماكن المُعادة لطلب التفاصيل لاحقًا بدل تحويل الأسماء نفسها مرارًا. اقرأ [استخدام CLI](./cli) للأتمتة أو [تكامل MCP](./mcp) لتوصيل وكيل. توضح [الوصفات](./recipes) الجمع بين الأدوات؛ ويوثق [المرجع الكامل](../reference/tools) كل معامل وقيمة افتراضية ونتيجة.

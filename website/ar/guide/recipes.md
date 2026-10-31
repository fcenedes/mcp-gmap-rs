# وصفات العمل

اجمع استدعاءات صغيرة وصريحة حول موقع واحد أو قائمة مختصرة من معرّفات الأماكن المُعادة. ترسل هذه الأمثلة طلبات Google قد تكون قابلة للفوترة. راجع [المصادقة](./authentication) أولًا؛ فقد تستخدم الأدوات المركبة عدة واجهات وتعيد أخطاء جزئية.

## العثور على مقهى ومقارنة الرحلة {#find-a-cafe-and-compare-the-trip}

ابحث ضمن نصف قطر فعلي، ثم وسّع معرّفات القائمة المختصرة فقط وقارن التنقل. معامل `keyword` في Nearby هو نوع مكان لدى Google، وليس نصًا حرًا.

```sh
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 500 --open-now --min-rating 4
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --max-photos 0
mcp-gmap-rs distance-matrix --origins '["48.8584,2.2945"]' \
  --destinations '["48.8606,2.3376","48.8530,2.3499"]' --mode walking
```

استخدم [`search-places`](../reference/tools#maps_search_places) للقصد باللغة الطبيعية، و[`compare-places`](../reference/tools#maps_compare_places) للمطابقات المفصلة مع مسافة قيادة اختيارية، أو [`explore-area`](../reference/tools#maps_explore_area) للاستكشاف بحسب الفئة. صفحة البحث ليست جردًا شاملًا. احتفظ بروابط مصادر Google ومؤلفي المراجعات وإفصاحات الذكاء الاصطناعي.

## تخطيط التوقفات والعثور على انعطاف {#plan-stops-and-find-a-detour}

```sh
mcp-gmap-rs plan-route \
  --stops '["48.8584,2.2945","48.8606,2.3376","48.8530,2.3499"]' \
  --mode walking --optimize false
mcp-gmap-rs search-along-route --text-query cafes \
  --origin '48.8584,2.2945' --destination '48.8606,2.3376' \
  --mode walking --max-results 3
```

تعالج [`directions`](../reference/tools#maps_directions) رحلة واحدة بخيارات الوقت والتجنب؛ وتحافظ [`plan-route`](../reference/tools#maps_plan_route) على النهايتين عند تحسين التوقفات الوسطية. يقبل النقل العام توقفين إجمالًا فقط. يتبع البحث على طول المسار خطًا متعدد النقاط بدل استخدام دائرة منتصف. وهو يفترض المشي، لذا اختر دائمًا وسيلة التنقل المقصودة.

## تحويل الموقع مرة وفحص الظروف المحلية {#resolve-once-inspect-local-conditions}

```sh
mcp-gmap-rs geocode --address 'Eiffel Tower, Paris, France'
mcp-gmap-rs weather --latitude 48.8584 --longitude 2.2945 \
  --type forecast_daily --forecast-days 3
mcp-gmap-rs air-quality --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs pollen --latitude 48.8584 --longitude 2.2945 --days 3
mcp-gmap-rs timezone --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs elevation --locations '[{"latitude":48.8584,"longitude":2.2945}]'
```

أعد استخدام الإحداثيات بدل تكرار التحويل. أبقِ الأرصاد والتوقعات وتوصيات المزوّد منفصلة؛ فقد تختلف التغطية. تحوّل [`reverse-geocode`](../reference/tools#maps_reverse_geocode) الإحداثيات إلى عنوان؛ وتحول [`batch-geocode`](../reference/tools#maps_batch_geocode) حتى 50 عنوانًا مع أخطاء لكل عنوان. تستخدم الطوابع الاختيارية في Time Zone مللي ثانية Unix؛ والارتفاع بالأمتار.

## التحقق من عنوان بريدي {#validate-a-postal-address}

```sh
mcp-gmap-rs validate-address --address-lines '["5 Avenue Anatole France"]' \
  --locality Paris --postal-code 75007 --region-code FR
```

افحص الحكم والتصحيحات والمكونات غير المحسومة. يختلف التحقق من العنوان عن الترميز الجغرافي ولا يضمن التوصيل. USPS CASS اختياري ويتطلب `regionCode` بقيمة US أو PR في هذا التنفيذ.

## عدّ الأماكن دون عدّ نتائج البحث {#count-places-without-counting-search-hits}

```sh
mcp-gmap-rs area-insights \
  --area '{"circle":{"latLng":{"latitude":48.8584,"longitude":2.2945},"radius":500}}' \
  --included-types '["restaurant"]' --min-rating 4
```

تستخدم الأعداد التجميعية عادة تمثيل int64 النصي من Google. تتوفر معرّفات الأماكن الاختيارية فقط عندما لا يزيد العدد على 100 ؛ ولا تُجلب التفاصيل تلقائيًا. لتحليل الظهور المطلوب صراحة، تقيس [`local-rank-tracker`](../reference/tools#maps_local_rank_tracker) ترتيب الكلمات المفتاحية عبر شبكة. ابدأ بكلمة واحدة وشبكة 3×3 ؛ فالترتيب ليس تقييم المراجعات، ويجب إظهار النقاط الفاشلة.

## مطابقة GPS وحفظ خريطة {#match-gps-data-and-save-a-map}

استخدم [`snap-to-roads`](../reference/tools#maps_snap_to_roads) لمسار مرتب و[`nearest-roads`](../reference/tools#maps_nearest_roads) لنقاط مستقلة. احتفظ بـ `originalIndex`؛ يضيف الاستيفاء نقاطًا قد لا تحتويه. لا تُعاد حدود السرعة.

```sh
mcp-gmap-rs nearest-roads --points '[{"latitude":48.8584,"longitude":2.2945}]'
mcp-gmap-rs static-map --center '48.8584,2.2945' --zoom 14 --output area.png
```

يسرد [مرجع الأدوات](../reference/tools) الحدود الدقيقة والحقول المطلوبة والقيم الافتراضية وصيغ النتائج لكل عملية.

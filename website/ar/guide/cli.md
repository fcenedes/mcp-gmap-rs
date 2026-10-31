# واجهة سطر الأوامر

استخدم CLI للاستعلامات المنفردة وأتمتة الصدفة والاستدعاءات المنظمة للعمليات الفرعية. تتشارك كل أداة مخطط المدخلات والتنفيذ مع MCP.

## اكتشاف الواجهة المثبتة {#discover-the-installed-interface}

```sh
mcp-gmap-rs --version
mcp-gmap-rs --help
mcp-gmap-rs tools
mcp-gmap-rs schema search-nearby
mcp-gmap-rs search-nearby --help
```

تعرض `tools` الأدوات المفعّلة ومخططاتها. افحص الإصدار المثبت بدل افتراض وجود كل إمكانات أحدث دليل. التشغيل دون أمر فرعي يبدأ خادم stdio، لذلك حدد أمرًا دائمًا للأتمتة المنفردة.

## الخيارات أو JSON المشترك {#flags-or-shared-json}

تستخدم أوامر الأدوات خيارات بصيغة kebab-case؛ وتأخذ قيم الكائنات والمصفوفات JSON. تقبل الخيارات المنطقية `true` أو `false`، ويعني الخيار المنطقي المجرد `true`.

```sh
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 500 --open-now
```

تقبل واجهة `call` العامة اسم أداة MCP المنطقي أو اسم أمر CLI، وتستخدم كائن معاملات MCP نفسه تمامًا:

```sh
mcp-gmap-rs call maps_geocode --args '{"address":"Paris, France"}'
printf '%s' '{"address":"Paris, France"}' | mcp-gmap-rs call geocode --args -
```

عند تشغيل عملية فرعية برمجيًا، مرّر JSON المُسلسل كمعامل مستقل. تجنّب إدراج نص غير موثوق داخل سلاسل أوامر الصدفة. لمعاملات المستوى الأعلى خيارات؛ وتبقى الحقول المتداخلة داخل كائنات JSON.

## المخرجات ومعالجة الإخفاقات {#output-and-failure-handling}

نتائج الأدوات العادية هي JSON على stdout. تذهب الأخطاء إلى stderr وتنتج حالة خروج غير صفرية. يكتب `doctor` تقرير JSON إلى stdout أيضًا عند إخفاق الفحوص. قد تحتوي النتائج الدفعية والمصفوفية والمركبة على أخطاء لكل عنصر حتى مع نجاح خروج الأمر؛ افحصها قبل اعتبار العملية كاملة.

```sh
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --output place.json
mcp-gmap-rs static-map --center Paris --zoom 12 --output paris.png
```

يكتب `--output` بصيغة JSON للأدوات العادية وPNG خام للخريطة الثابتة. ودونه، تحتوي مخرجات الخريطة الثابتة في CLI بيانات base64. تستخدم الأوقات عمومًا RFC 3339 مع منطقة زمنية صريحة؛ أما `timestamp` الاختياري لأداة Time Zone فيستخدم **مللي ثانية** Unix. راجع [كل الأدوات والأمثلة](../reference/tools).

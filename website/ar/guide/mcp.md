# تكامل MCP

وصّل مضيف MCP عبر stdio لعملية فرعية محلية أو Streamable HTTP لخدمة طويلة التشغيل. يعرض كلاهما فهرس الأدوات المفعّلة ومخططاتها نفسه.

## stdio محلي {#local-stdio}

مثال عام لإعداد عميل MCP:

```json
{
  "mcpServers": {
    "google-maps": {
      "command": "/absolute/path/to/mcp-gmap-rs",
      "args": ["--env-file", "/secure/mcp-gmap/runtime.env", "serve", "stdio"]
    }
  }
}
```

استخدم تنسيق إعدادات المضيف ومسارًا مطلقًا للملف التنفيذي. يجب وجود ملف dotenv المحدد. ويمكنك حقن متغيرات البيئة عبر المضيف بدلًا منه. هذا إعداد خادم MCP مستقل؛ ولا يلزم مكوّن تشغيل خاص بمساعد معين.

```sh
mcp-gmap-rs serve stdio
```

تحمل stdin وstdout رسائل MCP JSON-RPC مفصولة بأسطر فقط. تذهب رسائل التشخيص إلى stderr. تشغيل الملف التنفيذي دون معاملات يبدأ وسيلة النقل نفسها. يجب على مضيف stdio المبني على حاوية وصل stdin دون تخصيص TTY.

## Streamable HTTP {#streamable-http}

```sh
mcp-gmap-rs serve http --listen 127.0.0.1:3000
```

اضبط عنوان Streamable HTTP في عميل MCP إلى **`http://127.0.0.1:3000/mcp`**. عندما يدعم المضيف إدخالات MCP بعيدة، اضبط نقطة النهاية وترويسة التفويض:

```json
{
  "url": "https://maps.example.com/mcp",
  "headers": {
    "Authorization": "Bearer your-mcp-http-token"
  }
}
```

يعتمد الغلاف الدقيق لهذا الإدخال على مضيف MCP. يدعم الخادم تفاوض البروتوكول وعمليات الجلسة المناسبة POST وGET/SSE وDELETE عبر Rust MCP SDK. استخدم عميل MCP بدل التعامل مع طلب JSON-RPC POST واحد معزول بوصفه جلسة كاملة.

## عناوين الاستماع والوصول {#bind-addresses-and-access}

تتطلب المستمعات غير المحلية `MCP_HTTP_TOKEN`. يرسل العملاء هذا الرمز كاعتماد bearer، منفصلًا عن Google OAuth أو مفاتيح API. أضف قيمة Host لنقطة النهاية وأصول المتصفح الفعلية صراحة:

```sh
mcp-gmap-rs serve http --listen 0.0.0.0:3000 \
  --allowed-host maps.example.com \
  --allowed-origin https://app.example.com:443
```

يرفض الإعداد الافتراضي ترويسات Origin الموجودة ما لم تكن في قائمة السماح. الحد الأقصى لجسم الطلب 1 MiB. يفحص `/health` حياة العملية دون استدعاء Google. للوصول العام، اضبط [TLS/mTLS الأصلي أو بوابة](./production). يجب أن تحافظ الوكلاء على التفويض وترويسات بروتوكول MCP والجلسة وتسمح بالبث دون تخزين مؤقت. الجلسات وهوية Google محليتان للعملية؛ راجع [النشر](./deployment) قبل إضافة نسخ.

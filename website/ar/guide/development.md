# التطوير والإصدارات

بيئة التشغيل مشروع Rust مستقل. تتشارك CLI وMCP stdio وStreamable HTTP تنفيذ الأدوات نفسه. موقع التوثيق أدوات تأليف وبناء منفصلة؛ ولا يحتاج المستخدم النهائي إلا الملف التنفيذي المجمّع.

## البناء والفحص {#build-and-check}

استنسخ [المستودع](https://github.com/fcenedes/mcp-gmap-rs)، ثم استخدم ملف القفل الخاص به:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
cargo test --doc --all-features --locked
cargo build --release --locked
```

الحد الأدنى المعلن هو Rust 1.88 ؛ وتستخدم ملفات الإصدار Rust 1.94.1. تستخدم الاختبارات الافتراضية بدائل Google/الرموز المحلية ولا تحتاج بيانات اعتماد أو شبكة إلى Google. يفحص CI أيضًا Linux/macOS والحد الأدنى لـ Rust وrustdoc المستقر/الليلي وحزمة المصدر واختبارات Docker/MCP/TLS وملفات Kubernetes.

بعد التحقق المحلي، يستعيد `cargo clean` مساحة مخرجات البناء المولّدة؛ وتعاد عملية تجميعها في البناء اللاحق.

## الحفاظ على اكتمال المرجع {#keep-the-reference-complete}

يجمع دليل الأدوات المرجعي بين مخططات الملف التنفيذي الفعلية والقيم الافتراضية والنتائج وملاحظات السلوك التي تمت مراجعتها:

```sh
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs --check --verify-examples
```

يزيل الفاحص بيانات الاعتماد ويعطل dotenv ويتحقق من كل الأمثلة عند حد غياب المفتاح دون الاتصال بـ Google. حدّث ملاحظات JSON المُراجعة عند تغيير المعاملات أو السلوك.

الاختبارات الحية اختيارية وقد تكون قابلة للفوترة. بمفتاح اختبار مضبوط صراحة، شغّل `cargo test --locked --test live live_maps_ -- --ignored --test-threads=1`. تستخدم اختبارات OAuth الستة مرشح `live_oauth_` وتزيل المفتاح من العمليات الفرعية. لا تحفظ بيانات الاعتماد ولا تفعّل الاختبارات الحية في CI المعتاد.

## نشر إصدار {#publish-a-version}

حدّث إصدار Cargo وملف القفل وسجل التغييرات وأمثلة التثبيت وملاحظات الإصدار معًا. ادفع التغييرات وانتظر CI، ثم أنشئ وسمًا موضحًا مطابقًا بصيغة `vX.Y.Z`. يفحص سير Release الوسم نفسه ويبني ثلاثة أهداف أصلية ويتحقق من الأرشيفات والبصمات وينشر المسودة المرحلية.

يتطلب سير **Publish crate** المستقل إصدار GitHub المنشور ذاك. شغّله للوسم نفسه. خزّن `CARGO_REGISTRY_TOKEN` المحدود النطاق في بيئة GitHub المسماة `crates-io`، وليس في المستودع. يبني docs.rs تلقائيًا بعد النشر إلى crates.io؛ ولا يحدّث إصدار GitHub وحده توثيق Rust المستضاف. تحقّق من البناء المستضاف فعليًا بعد الرفع.

راجع [إجراءات الإصدار](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/releases.md) و[سجل التحقق](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/validation.md) و[نقطة دخول Rust العامة](https://docs.rs/mcp-gmap-rs/latest/mcp_gmap_rs/fn.run.html).

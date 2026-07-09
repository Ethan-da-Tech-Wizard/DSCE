# DSCE Dependency Roadmap — Library & Framework Vials To Be Added

> **Status: living document — more to come.** This list will keep growing as
> new vials are authored and new capability vocabulary is added to the
> harvester. Nothing here is final; entries are added, split, and promoted
> to vials incrementally.

This roadmap enumerates the libraries, frameworks, and toolchains the
synthesis knowledge base (`vials_synthesis/`) should learn next, so that
DSCE can assemble **complex applications** — not just CLI utilities — and
so that **every target language can produce GUI applications and web
apps**.

## Authoring rules (apply to every entry below)

1. **Reliable sources only.** The engine is deterministic; its knowledge
   must be too. Every vial's `evidence` field cites the authoritative
   source: the ISO/ECMA standard, the language's official standard-library
   reference, or the framework's official documentation site. No blog
   posts, no Stack Overflow, no paraphrased tutorials.
2. **One vial per library, pure data.** Library vials contain only facts
   (API snippets, `provides`, `implements_language`, `depends_on`,
   `install_command`, build/run metadata). Assembly logic lives in pattern
   vials, which never name a concrete library.
3. **Capabilities are abstract.** The harvester maps request keywords to
   capability names (`needs`); vials declare `provides`. New capabilities
   proposed here: `gui`, `web_server`, `web_frontend`, `http_client`,
   `orm`, `async_runtime`, `serialization`, `cli_args`, `logging`,
   `testing`, `audio`, `image_processing`, `crypto`, `compression`,
   `message_queue`, `cache`, `auth`, `templating`, `realtime`,
   `embedded_db`, `plotting`, `game_engine`, `pdf`, `email`, `scheduling`.
4. **Dependencies are facts.** Every `depends_on` edge below becomes a
   Datalog fact so `dependency_resolution` can compute transitive closures
   and install plans.

Legend: ✅ vial exists · 🔜 planned (this document)

---

## 1. Python (`lang_python`)

### GUI
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| tkinter ✅ | graphics, gui | Tcl/Tk | docs.python.org/3/library/tkinter |
| pygame ✅ | graphics, game_engine | SDL2 | pygame.org/docs |
| PyQt6 🔜 | gui | Qt 6 | riverbankcomputing.com/static/Docs/PyQt6 |
| PySide6 🔜 | gui | Qt 6 | doc.qt.io/qtforpython |
| Kivy 🔜 | gui (touch/mobile) | SDL2, OpenGL | kivy.org/doc |
| wxPython 🔜 | gui | wxWidgets | docs.wxpython.org |
| DearPyGui 🔜 | gui (immediate mode) | — | dearpygui.readthedocs.io |

### Web
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| Flask 🔜 | web_server, templating | Werkzeug, Jinja2 | flask.palletsprojects.com |
| Django 🔜 | web_server, orm, auth, templating | — | docs.djangoproject.com |
| FastAPI 🔜 | web_server, serialization | Starlette, Pydantic, uvicorn | fastapi.tiangolo.com |
| aiohttp 🔜 | web_server, http_client, async_runtime | asyncio | docs.aiohttp.org |
| requests 🔜 | http_client | urllib3 | requests.readthedocs.io |
| httpx 🔜 | http_client (async) | — | python-httpx.org |
| Jinja2 🔜 | templating | — | jinja.palletsprojects.com |
| gunicorn 🔜 | web_server (WSGI runner) | — | docs.gunicorn.org |
| uvicorn 🔜 | web_server (ASGI runner) | — | uvicorn.org |

### Data / scientific / complex apps
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| NumPy 🔜 | tensor_computation (arrays) | — | numpy.org/doc |
| pandas 🔜 | dataframes | NumPy | pandas.pydata.org/docs |
| matplotlib 🔜 | plotting | NumPy | matplotlib.org/stable |
| SQLAlchemy 🔜 | orm | — | docs.sqlalchemy.org |
| pydantic 🔜 | serialization | — | docs.pydantic.dev |
| Pillow 🔜 | image_processing | — | pillow.readthedocs.io |
| celery 🔜 | message_queue, scheduling | Redis/RabbitMQ | docs.celeryq.dev |
| pytest 🔜 | testing | — | docs.pytest.org |
| cryptography 🔜 | crypto | OpenSSL | cryptography.io |
| TensorFlow 🔜 | tensor_computation | python_runtime | tensorflow.org/api_docs |
| scikit-learn 🔜 | machine_learning | NumPy, SciPy | scikit-learn.org/stable |
| transformers 🔜 | machine_learning (NLP) | PyTorch | huggingface.co/docs/transformers |
| PyTorch ✅ | tensor_computation | python_runtime | pytorch.org/docs |
| FAISS ✅ | vector_search | NumPy | github.com/facebookresearch/faiss/wiki |

## 2. Rust (`lang_rust`)

### GUI
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| egui/eframe 🔜 | gui (immediate mode) | winit, wgpu | docs.rs/egui |
| iced 🔜 | gui (Elm-style) | winit, wgpu | docs.rs/iced |
| Tauri 🔜 | desktop_web | system WebView, Node.js (tooling) | tauri.app |
| gtk4-rs 🔜 | gui | GTK 4 | gtk-rs.org/gtk4-rs/stable/latest/docs |
| Slint 🔜 | gui (declarative) | — | slint.dev/docs |
| bevy 🔜 | game_engine | wgpu | bevyengine.org/learn |

### Web
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| axum 🔜 | web_server | tokio, hyper, tower | docs.rs/axum |
| actix-web 🔜 | web_server | tokio | actix.rs/docs |
| rocket 🔜 | web_server | tokio | rocket.rs/guide |
| warp 🔜 | web_server | tokio, hyper | docs.rs/warp |
| reqwest 🔜 | http_client | tokio, hyper | docs.rs/reqwest |
| hyper 🔜 | http_client, web_server (low level) | tokio | hyper.rs/guides |
| Yew 🔜 | web_frontend (WASM) | wasm-bindgen, trunk | yew.rs/docs |
| Leptos 🔜 | web_frontend (WASM) | wasm-bindgen | leptos.dev |

### Core ecosystem
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| tokio 🔜 | async_runtime | — | tokio.rs / docs.rs/tokio |
| serde + serde_json 🔜 | serialization | — | serde.rs |
| clap 🔜 | cli_args | — | docs.rs/clap |
| rand 🔜 | random_generation | — | docs.rs/rand |
| sqlx 🔜 | orm (async SQL) | tokio | docs.rs/sqlx |
| diesel 🔜 | orm | — | diesel.rs/guides |
| tracing 🔜 | logging | — | docs.rs/tracing |
| rayon 🔜 | parallelism | — | docs.rs/rayon |

## 3. C (`lang_c`)

### GUI
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| GTK 4 🔜 | gui | GLib, cairo, Pango | docs.gtk.org/gtk4 |
| SDL2 🔜 | graphics, game_engine | — | wiki.libsdl.org |
| raylib 🔜 | graphics, game_engine | — | raylib.com/cheatsheet |
| Nuklear 🔜 | gui (immediate mode) | — | github.com/Immediate-Mode-UI/Nuklear (docs) |
| ncurses 🔜 | tui (terminal UI) | — | invisible-island.net/ncurses |

### Web / network
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| libmicrohttpd 🔜 | web_server | — | gnu.org/software/libmicrohttpd |
| libcurl 🔜 | http_client | OpenSSL | curl.se/libcurl |
| civetweb 🔜 | web_server | — | civetweb.github.io/civetweb |
| mongoose 🔜 | web_server, realtime | — | mongoose.ws/documentation |
| OpenSSL 🔜 | crypto | — | openssl.org/docs |
| SQLite C API ✅/🔜 | embedded_db | — | sqlite.org/cintro.html |
| cJSON 🔜 | serialization | — | github.com/DaveGamble/cJSON (docs) |
| zlib 🔜 | compression | — | zlib.net/manual.html |

## 4. C++ (`lang_cpp`)

### GUI
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| Qt Widgets ✅ | gui, cpp_graphics | Qt 5/6 | doc.qt.io |
| Qt Quick/QML 🔜 | gui (declarative) | Qt 6 | doc.qt.io/qt-6/qtquick-index.html |
| wxWidgets 🔜 | gui | — | docs.wxwidgets.org |
| Dear ImGui 🔜 | gui (immediate mode) | GLFW/SDL2 backend | github.com/ocornut/imgui (docs) |
| FLTK 🔜 | gui | — | fltk.org/documentation.php |
| JUCE 🔜 | gui, audio | — | juce.com/learn/documentation |
| SFML 🔜 | graphics, game_engine, audio | — | sfml-dev.org/documentation |
| OpenGL/GLFW 🔜 | graphics (low level) | — | glfw.org/docs, khronos.org/opengl |

### Web
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| Crow 🔜 | web_server | Asio | crowcpp.org |
| Drogon 🔜 | web_server, orm | — | drogon.org/docs |
| cpp-httplib 🔜 | web_server, http_client | — | github.com/yhirose/cpp-httplib (docs) |
| Boost.Beast 🔜 | web_server, realtime (WebSocket) | Boost.Asio | boost.org/doc/libs (beast) |
| gRPC 🔜 | rpc | Protocol Buffers | grpc.io/docs |

### Core ecosystem
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| Boost 🔜 | (many: asio, filesystem…) | — | boost.org/doc |
| nlohmann/json 🔜 | serialization | — | json.nlohmann.me |
| fmt 🔜 | formatting | — | fmt.dev |
| spdlog 🔜 | logging | fmt | github.com/gabime/spdlog (docs) |
| Eigen 🔜 | tensor_computation | — | eigen.tuxfamily.org/dox |
| OpenCV 🔜 | image_processing | — | docs.opencv.org |
| Catch2 / GoogleTest 🔜 | testing | — | github.com/catchorg/Catch2, google.github.io/googletest |
| CMake 🔜 | build_system | — | cmake.org/documentation |

## 5. C# (`lang_csharp`)

### GUI
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| WinForms 🔜 | gui (Windows) | .NET | learn.microsoft.com/dotnet/desktop/winforms |
| WPF 🔜 | gui (Windows, XAML) | .NET | learn.microsoft.com/dotnet/desktop/wpf |
| MAUI 🔜 | gui (cross-platform/mobile) | .NET | learn.microsoft.com/dotnet/maui |
| Avalonia 🔜 | gui (cross-platform, XAML) | .NET | docs.avaloniaui.net |
| Uno Platform 🔜 | gui (cross-platform) | .NET | platform.uno/docs |

### Web
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| ASP.NET Core 🔜 | web_server, auth | .NET | learn.microsoft.com/aspnet/core |
| Blazor 🔜 | web_frontend (WASM/server) | ASP.NET Core | learn.microsoft.com/aspnet/core/blazor |
| SignalR 🔜 | realtime | ASP.NET Core | learn.microsoft.com/aspnet/core/signalr |
| Entity Framework Core 🔜 | orm | .NET | learn.microsoft.com/ef/core |
| HttpClient (BCL) 🔜 | http_client | .NET | learn.microsoft.com/dotnet/api/system.net.http.httpclient |
| Newtonsoft.Json / System.Text.Json 🔜 | serialization | — | learn.microsoft.com/dotnet/standard/serialization |
| xUnit / NUnit 🔜 | testing | — | xunit.net, docs.nunit.org |

## 6. Go (`lang_go`)

### GUI
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| Fyne 🔜 | gui | OpenGL | docs.fyne.io |
| Gio 🔜 | gui (immediate mode) | — | gioui.org/doc |
| Wails 🔜 | desktop_web | system WebView | wails.io/docs |
| Ebitengine 🔜 | game_engine | — | ebitengine.org/en/documents |

### Web
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| net/http (stdlib) 🔜 | web_server, http_client | — | pkg.go.dev/net/http |
| html/template (stdlib) 🔜 | templating | — | pkg.go.dev/html/template |
| Gin 🔜 | web_server | net/http | gin-gonic.com/docs |
| Echo 🔜 | web_server | net/http | echo.labstack.com/docs |
| Fiber 🔜 | web_server | fasthttp | docs.gofiber.io |
| chi 🔜 | web_server (router) | net/http | pkg.go.dev/github.com/go-chi/chi |
| gorilla/websocket 🔜 | realtime | net/http | pkg.go.dev/github.com/gorilla/websocket |
| GORM 🔜 | orm | — | gorm.io/docs |
| encoding/json (stdlib) 🔜 | serialization | — | pkg.go.dev/encoding/json |
| cobra 🔜 | cli_args | — | pkg.go.dev/github.com/spf13/cobra |
| gRPC-Go 🔜 | rpc | Protocol Buffers | grpc.io/docs/languages/go |

## 7. JavaScript / TypeScript (`lang_javascript`)

### GUI (desktop)
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| Electron ✅ | desktop_web | Node.js, Chromium | electronjs.org/docs |
| NW.js 🔜 | desktop_web | Node.js, Chromium | docs.nwjs.io |

### Web frontend
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| React ✅ | web_ui | react-dom, Node.js (tooling) | react.dev |
| Vue 🔜 | web_frontend | Node.js (tooling) | vuejs.org/guide |
| Angular 🔜 | web_frontend | Node.js, TypeScript, RxJS | angular.dev |
| Svelte / SvelteKit 🔜 | web_frontend | Node.js, Vite | svelte.dev/docs |
| Next.js 🔜 | web_frontend, web_server (SSR) | React, Node.js | nextjs.org/docs |
| Vite 🔜 | build_system | Node.js, esbuild, Rollup | vite.dev/guide |
| Tailwind CSS 🔜 | styling | Node.js (tooling) | tailwindcss.com/docs |
| D3.js 🔜 | plotting | — | d3js.org |
| Three.js 🔜 | graphics (WebGL 3D) | — | threejs.org/docs |

### Web backend (Node.js)
| Library | Provides | Depends on | Source to cite |
|---|---|---|---|
| node:http (stdlib) 🔜 | web_server | Node.js | nodejs.org/api/http.html |
| Express 🔜 | web_server | Node.js | expressjs.com |
| Fastify 🔜 | web_server | Node.js | fastify.dev/docs |
| NestJS 🔜 | web_server (structured) | Node.js, TypeScript | docs.nestjs.com |
| Socket.IO 🔜 | realtime | Node.js | socket.io/docs |
| ws 🔜 | realtime (WebSocket) | Node.js | github.com/websockets/ws (docs) |
| Prisma 🔜 | orm | Node.js | prisma.io/docs |
| TypeScript 🔜 | language tooling | Node.js | typescriptlang.org/docs |
| Jest / Vitest 🔜 | testing | Node.js | jestjs.io/docs, vitest.dev |

## 8. SQL (`lang_sql`)

SQL is both a target language and the persistence layer of complex apps.

| Engine / tool | Provides | Depends on | Source to cite |
|---|---|---|---|
| SQLite ✅ | embedded_db, persistence | — | sqlite.org/docs.html |
| PostgreSQL 🔜 | database, persistence | — | postgresql.org/docs |
| MySQL / MariaDB 🔜 | database | — | dev.mysql.com/doc, mariadb.com/kb |
| Microsoft SQL Server (T-SQL) 🔜 | database | — | learn.microsoft.com/sql |
| DDL/DML pattern vials 🔜 | schema synthesis (CREATE TABLE, JOIN, index, view, trigger) | — | ISO/IEC 9075 + engine docs |
| Redis 🔜 | cache, message_queue | — | redis.io/docs |
| MongoDB 🔜 | database (document) | — | mongodb.com/docs |

GUI/web note: SQL has no native GUI — the pattern vials pair `lang_sql`
schema output with a host language's `orm`/`gui`/`web_server` vials
(e.g. SQLite schema + Python Flask + SQLAlchemy in one assembled app).

## 9. Assembly (`lang_assembly`)

Assembly "GUI/web" means binding to system libraries and syscall surfaces;
these vials document calling conventions rather than widget toolkits.

| Item | Provides | Depends on | Source to cite |
|---|---|---|---|
| Linux syscall ABI ✅/🔜 | io (expand: sockets, mmap, futex) | — | System V AMD64 ABI, man7.org syscalls(2) |
| C library FFI pattern 🔜 | calling libc/GTK/SDL from asm | libc | System V AMD64 ABI calling convention |
| Win32 API (MASM) 🔜 | gui (MessageBox, CreateWindowEx) | kernel32, user32 | learn.microsoft.com/windows/win32 |
| BIOS/UEFI bare metal 🔜 | boot_code | — | UEFI specification (uefi.org) |
| SIMD (SSE/AVX) 🔜 | vector math | — | Intel SDM / Intel Intrinsics Guide |
| Sockets via syscalls 🔜 | web_server (minimal HTTP) | — | man7.org socket(2), accept(2) |
| WebAssembly (WAT) 🔜 | web target for asm | — | webassembly.github.io/spec |

## 10. Cross-cutting infrastructure (language-independent vials)

| Item | Provides | Depends on | Source to cite |
|---|---|---|---|
| Docker ✅ | containerization | containerd → runc | docs.docker.com |
| Docker Compose 🔜 | multi-service orchestration (dev) | Docker | docs.docker.com/compose |
| Kubernetes ✅ | orchestration | container_runtime | kubernetes.io/docs |
| Helm 🔜 | k8s packaging | Kubernetes | helm.sh/docs |
| Terraform 🔜 | infrastructure_as_code | — | developer.hashicorp.com/terraform/docs |
| Ansible 🔜 | configuration management | Python | docs.ansible.com |
| nginx 🔜 | web_server (reverse proxy) | — | nginx.org/en/docs |
| Apache Kafka 🔜 | message_queue | JVM | kafka.apache.org/documentation |
| RabbitMQ 🔜 | message_queue | Erlang/OTP | rabbitmq.com/docs |
| GraphQL 🔜 | api_schema | — | graphql.org/learn + spec.graphql.org |
| OpenAPI/Swagger 🔜 | api_schema | — | spec.openapis.org |
| Protocol Buffers 🔜 | serialization, rpc | — | protobuf.dev |
| OAuth 2.0 / OIDC 🔜 | auth | — | RFC 6749, openid.net/specs |
| JSON Web Tokens 🔜 | auth | — | RFC 7519 |
| TLS/HTTPS 🔜 | crypto (transport) | OpenSSL et al. | RFC 8446 |
| WebSocket protocol 🔜 | realtime | — | RFC 6455 |
| HTTP semantics 🔜 | web fundamentals | — | RFC 9110–9114 |
| CUDA ✅ | gpu_acceleration | nvidia_driver | docs.nvidia.com/cuda |
| Vulkan 🔜 | graphics (low level) | GPU driver | registry.khronos.org/vulkan |
| OpenCL 🔜 | gpu_acceleration | — | registry.khronos.org/OpenCL |
| Git 🔜 | version_control | — | git-scm.com/docs |
| GitHub Actions 🔜 | ci_cd | — | docs.github.com/actions |

## 11. Pattern vials required to use all of the above

Libraries alone don't assemble apps; each capability family needs a
language-agnostic pattern vial (like `polyglot_cli_app`):

- 🔜 `polyglot_gui_app` — binds `needs gui` + `target_language` to any
  vial that `provides gui` and `implements_language` the target; assembles
  window/widget/event-loop sections per the GUI vial's documented skeleton.
- 🔜 `polyglot_web_server` — route + handler + listener skeleton; binds
  `needs web_server` per language.
- 🔜 `polyglot_web_frontend` — component + render-root skeleton
  (React/Vue/Blazor/Yew all share this shape).
- 🔜 `client_server_app` — composes a `web_server` app with an
  `http_client`/`web_frontend` app in one request.
- 🔜 `full_stack_app` — model (orm/database) + API (web_server) +
  frontend (web_frontend) + deployment (containerization) in one flood.
- 🔜 `test_harness` — pairs any assembled app with its language's testing
  vial.
- 🔜 `build_manifest` — emits the package manifest alongside the code
  (Cargo.toml, package.json, go.mod, pyproject.toml, .csproj, CMakeLists),
  each cited from the official manifest reference.

## What "more to come" means concretely

- Mobile targets (Swift/SwiftUI, Kotlin/Jetpack Compose, Flutter/Dart).
- More languages: Java (Spring, JavaFX), Ruby (Rails), PHP (Laravel),
  Zig, Haskell, Lua, R, Julia, Scala, Kotlin, Swift.
- Bulk importers (like `scripts/download_dictionary.py`) that convert
  official API references into library vials mechanically, so coverage
  grows by the thousands of symbols instead of hand-written snippets.
- Version pinning facts (`requires_version`, `min_language_version`) so
  dependency resolution can flag incompatible combinations as conflicts
  via functional predicates.

*This document is a roadmap, not a promise of order — vials are promoted
from 🔜 to ✅ as they are authored, tested, and their synthesized output
verified against real toolchains.*
